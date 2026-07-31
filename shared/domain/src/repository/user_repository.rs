use async_trait::async_trait;
use crate::models::user::AuthUser;
use crate::errors::DomainError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_pan(&self, pan: &str) -> Result<Option<AuthUser>, DomainError>;
    async fn mark_login(&self, uuid: &str) -> Result<(), DomainError>;
}