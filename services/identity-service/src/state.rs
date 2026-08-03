use crate::config::IdentityConfig;

pub struct AppState {
    pub config: IdentityConfig,
    // pub db: DatabaseConnection,  // wire up later
    // pub cache: RedisCache,       // wire up later
}
