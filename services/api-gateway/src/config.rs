use config::env_vars::Environment;
use config::loader;

pub struct GatewayConfig {
    pub env: Environment,
    pub identity_service_url: String,
    pub payment_service_url: String,
    pub rate_limit_rpm: u32,
    pub host: String,
    pub port: u16,
    pub is_vapt: bool,
    pub aes_secret: String,
    pub aes_hash_key: String,
}

impl GatewayConfig {
    pub fn load() -> Self {
        let env = Environment::detect();
        loader::load_env_file("api-gateway");

        Self {
            env,
            identity_service_url: loader::require_var("IDENTITY_SERVICE_URL"),
            payment_service_url: loader::require_var("PAYMENT_SERVICE_URL"),
            rate_limit_rpm: loader::optional_var("RATE_LIMIT_RPM", "100")
                .parse()
                .expect("RATE_LIMIT_RPM must be a number"),
            host: loader::optional_var("HOST", "0.0.0.0"),
            port: loader::optional_var("PORT", "8080")
                .parse()
                .expect("PORT must be a number"),
            is_vapt: loader::optional_var("VAPT", "no").eq_ignore_ascii_case("yes"),
            aes_secret: loader::require_var("AES_SECRET"),
            aes_hash_key: loader::require_var("AES_HASH_KEY"),
        }
    }
}
