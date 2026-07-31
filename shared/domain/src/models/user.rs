use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub pan: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    #[serde(flatten)]
    pub base: User,
    pub password_hash: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub login_flag: bool,
}

impl AuthUser {
    pub fn uuid(&self) -> &str { &self.base.uuid }
    pub fn pan(&self) -> &str { &self.base.pan }
}