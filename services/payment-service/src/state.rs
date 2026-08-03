use crate::config::PaymentConfig;

pub struct AppState {
    pub config: PaymentConfig,
    // pub db: DatabaseConnection,  // wire up later
    // pub cache: RedisCache,       // wire up later
}
