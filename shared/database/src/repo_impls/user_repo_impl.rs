use async_trait::async_trait;
use sea_orm::*;
use domain::repository::user_repository::UserRepository;
use domain::models::user::{User, AuthUser};
use domain::errors::DomainError;
use crate::entities::user;

pub struct UserRepoImpl {
    pub db: DatabaseConnection,
}

#[async_trait]
impl UserRepository for UserRepoImpl {
    async fn find_by_pan(&self, pan: &str) -> Result<Option<AuthUser>, DomainError> {
        let row = user::Entity::find()
            .filter(user::Column::Pan.eq(pan))
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(row.map(|m| AuthUser {
            base: User {
                uuid: m.uuid,
                pan: m.pan,
                name: m.name,
            },
            password_hash: m.password,
            email: m.email,
            mobile: m.mobile,
            login_flag: m.login_flag != 0,
        }))
    }

    async fn mark_login(&self, uuid: &str) -> Result<(), DomainError> {
        let mut active: user::ActiveModel = user::Entity::find_by_id(uuid.to_string())
            .one(&self.db)
            .await
            .map_err(|e| DomainError::InternalError(e.to_string()))?
            .ok_or(DomainError::NotFound)?
            .into();

        active.login_flag = Set(1);
        active.update(&self.db).await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
            
        Ok(())
    }
}
