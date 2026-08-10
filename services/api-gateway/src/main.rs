mod config;
mod constants;
mod routes;
mod state;

use crate::config::GatewayConfig;

fn main() {
    let config = GatewayConfig::load();
    println!(
        "Starting API Gateway on {}:{} [{}]",
        config.host, config.port, config.env
    );

    println!("API Gateway is ready!");
}
