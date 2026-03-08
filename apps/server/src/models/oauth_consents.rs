use sea_orm::prelude::*;

use super::_entities::oauth_consents::{self};
pub use super::_entities::oauth_consents::{ActiveModel, Column, Entity, Model};

impl Model {
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }

    pub fn scopes_list(&self) -> Vec<String> {
        self.scopes
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Entity {
    pub async fn find_active_consent(
        db: &DatabaseConnection,
        app_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Self::find()
            .filter(
                sea_orm::Condition::all()
                    .add(oauth_consents::Column::AppId.eq(app_id))
                    .add(oauth_consents::Column::UserId.eq(user_id))
                    .add(oauth_consents::Column::RevokedAt.is_null()),
            )
            .one(db)
            .await
    }
}
