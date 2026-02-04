use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

use crate::auth::error::AuthError;
use crate::auth::user::{ActiveModel, Column, Entity, Model};

#[derive(Clone)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, user: Model) -> Result<Model, AuthError> {
        let active: ActiveModel = user.into();
        active.insert(&self.db).await.map_err(map_db_error)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<Model>, AuthError> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(map_db_error)
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Model>, AuthError> {
        Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(map_db_error)
    }

    pub async fn update_last_login(&self, id: uuid::Uuid) -> Result<(), AuthError> {
        let result = Entity::update_many()
            .filter(Column::Id.eq(id))
            .col_expr(
                Column::LastLoginAt,
                sea_orm::sea_query::Expr::current_timestamp(),
            )
            .exec(&self.db)
            .await
            .map_err(map_db_error)?;

        if result.rows_affected == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }
}

fn map_db_error(err: DbErr) -> AuthError {
    let message = err.to_string();
    if message.to_lowercase().contains("unique") {
        AuthError::EmailAlreadyExists
    } else {
        AuthError::Database(message)
    }
}
