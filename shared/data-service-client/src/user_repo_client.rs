use domain::repository::user_repository::UserRepository;
use domain::models::user::{AuthUser, User};
use domain::errors::DomainError;

pub struct RemoteUserRepo {
    pub http: reqwest::Client,
    pub base_url: String,
}

#[async_trait::async_trait]
impl UserRepository for RemoteUserRepo {
    
    async fn find_auth_by_pan(&self, pan: &str) -> Result<Option<AuthUser>, DomainError> {
        let url = format!("{}/internal/auth/by-pan/{}", self.base_url, pan);
        let resp = self.http.get(&url).send().await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        if resp.status() == 404 {
            return Ok(None);
        }

        // Deserializes directly into domain::models::user::AuthUser
        let auth_user: AuthUser = resp.json().await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;

        Ok(Some(auth_user))
    }

    async fn mark_login(&self, uuid: &str) -> Result<(), DomainError> {
        let url = format!("{}/internal/auth/{}/mark-login", self.base_url, uuid);
        self.http.post(&url).send().await
            .map_err(|e| DomainError::InternalError(e.to_string()))?;
        Ok(())
    }
}