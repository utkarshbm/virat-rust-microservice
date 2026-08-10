use crate::config::GatewayConfig;

pub struct AppState {
    pub config: GatewayConfig,
    // upstream clients, rate limiter state, etc. — wire up later
}
