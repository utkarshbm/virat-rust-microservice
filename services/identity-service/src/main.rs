mod application;
mod config;
mod constants;
mod dto;
mod handlers;
mod middleware;
mod state;
use logging::info;

use crate::config::IdentityConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guards = logging::init_tracing("identity-service");
    let config = IdentityConfig::load();
    info!(env = config.env.to_string(), "Environment loaded");

    println!(
        "Starting Identity Service on {}:{} [{}]",
        config.host, config.port, config.env
    );

    // 1. Establish direct database connection
    // let db = Database::connect(&config.database_url).await?;
    // println!("Database connected: {}", config.database_url);

    // 2. Initialize the Repository
    // let user_repo = UserRepoImpl { db };

    // 3. We can now use user_repo directly (e.g. user_repo.find_by_pan(...).await)

    println!("Identity Service is ready!");
    Ok(())
}
