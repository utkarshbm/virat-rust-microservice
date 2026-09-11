mod config;
mod constants;
mod routes;
mod state;

use crate::config::GatewayConfig;
use logging::info;

fn main() {
    // 1. Initialize structured logging.
    // The `_guards` variable must be bound to prevent the non-blocking file appenders from dropping!
    let _guards = logging::init_tracing("api-gateway");

    // 2. Load environment configuration
    let config = GatewayConfig::load();

    info!(
        host = %config.host,
        port = %config.port,
        env = %config.env,
        "Starting API Gateway"
    );

    info!("API Gateway initialized and ready for routing");
}
