use config::env_vars::Environment;
use config::loader;

pub struct IdentityConfig {
    pub env: Environment,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_secs: u64,
    pub host: String,
    pub port: u16,
}

impl IdentityConfig {
    pub fn load() -> Self {
        let env = Environment::detect();
        loader::load_env_file("identity-service");

        Self {
            env,
            database_url: loader::require_var("DATABASE_URL"),
            database_max_connections: loader::optional_var("DATABASE_MAX_CONNECTIONS", "10")
                .parse()
                .expect("DATABASE_MAX_CONNECTIONS must be a number"),
            redis_url: loader::require_var("REDIS_URL"),
            jwt_secret: loader::require_var("JWT_SECRET"),
            jwt_expiry_secs: loader::optional_var("JWT_EXPIRY_SECS", "3600")
                .parse()
                .expect("JWT_EXPIRY_SECS must be a number"),
            host: loader::optional_var("HOST", "0.0.0.0"),
            port: loader::optional_var("PORT", "8081")
                .parse()
                .expect("PORT must be a number"),
        }
    }
}
