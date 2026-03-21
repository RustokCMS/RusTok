use chrono::Utc;
use sea_orm::{entity::prelude::*, Condition, QueryFilter, QueryOrder};

pub use super::_entities::mcp_tokens::{ActiveModel, Column, Entity, Model, Relation};

impl Entity {
    pub async fn find_active_by_hash(
        db: &DatabaseConnection,
        token_hash: &str,
    ) -> Result<Option<Model>, DbErr> {
        Self::find()
            .filter(
                Condition::all()
                    .add(Column::TokenHash.eq(token_hash))
                    .add(Column::RevokedAt.is_null())
                    .add(
                        Condition::any()
                            .add(Column::ExpiresAt.is_null())
                            .add(Column::ExpiresAt.gt(Utc::now())),
                    ),
            )
            .one(db)
            .await
    }

    pub async fn find_by_client(
        db: &DatabaseConnection,
        client_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        Self::find()
            .filter(Column::ClientId.eq(client_id))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await
    }
}

impl Model {
    pub fn is_active(&self) -> bool {
        let not_revoked = self.revoked_at.is_none();
        let not_expired = self
            .expires_at
            .map(|expires_at| {
                let expires_at_utc: chrono::DateTime<Utc> = expires_at.into();
                expires_at_utc > Utc::now()
            })
            .unwrap_or(true);
        not_revoked && not_expired
    }
}
