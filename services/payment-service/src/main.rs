mod application;
mod config;
mod dto;
mod handlers;
mod integrations;
mod state;

use crate::config::PaymentConfig;

fn main() {
    let config = PaymentConfig::load();
    println!("Starting Payment Service on {}:{} [{}]", config.host, config.port, config.env);

    println!("Payment Service is ready!");
}
