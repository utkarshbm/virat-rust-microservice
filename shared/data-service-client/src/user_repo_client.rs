use domain::repository::user_repository::UserRepository;
use domain::models::user::User;
use domain::errors::DomainError;

pub struct RemoteUserRepo {
    pub http: reqwest::Client,
    pub base_url: String,
}

#[async_trait::async_trait]
impl UserRepository for RemoteUserRepo {
    async fn find_by_pan(&self, pan: &str) -> Result<Option<User>, DomainError> {
        let resp = self.http
            .get(format!("{}/internal/users/by-pan/{}", self.base_url, pan))
            .send().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if resp.status() == 404 { return Ok(None); }
        Ok(Some(resp.json::<User>().await.map_err(|e| DomainError::Internal(e.to_string()))?))
    }

    async fn mark_login(&self, uuid: &str) -> Result<(), DomainError> {
        self.http.post(format!("{}/internal/users/{}/mark-login", self.base_url, uuid))
            .send().await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        Ok(())
    }
}