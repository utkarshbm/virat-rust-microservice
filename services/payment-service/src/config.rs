use config::env_vars::Environment;
use config::loader;

pub struct PaymentConfig {
    pub env: Environment,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub razorpay_key_id: String,
    pub razorpay_key_secret: String,
    pub webhook_secret: String,
    pub host: String,
    pub port: u16,
}

impl PaymentConfig {
    pub fn load() -> Self {
        let env = Environment::detect();
        loader::load_env_file("payment-service");

        Self {
            env,
            database_url: loader::require_var("DATABASE_URL"),
            database_max_connections: loader::optional_var("DATABASE_MAX_CONNECTIONS", "10")
                .parse()
                .expect("DATABASE_MAX_CONNECTIONS must be a number"),
            redis_url: loader::require_var("REDIS_URL"),
            razorpay_key_id: loader::require_var("RAZORPAY_KEY_ID"),
            razorpay_key_secret: loader::require_var("RAZORPAY_KEY_SECRET"),
            webhook_secret: loader::require_var("WEBHOOK_SECRET"),
            host: loader::optional_var("HOST", "0.0.0.0"),
            port: loader::optional_var("PORT", "8082")
                .parse()
                .expect("PORT must be a number"),
        }
    }
}
