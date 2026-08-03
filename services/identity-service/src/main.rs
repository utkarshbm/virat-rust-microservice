use database::sea_orm::Database;
use database::repo_impls::user::UserRepoImpl;
use domain::repository::user::UserRepository;

mod application;
mod config;
mod constants;
mod dto;
mod handlers;
mod middleware;
mod state;

use crate::config::IdentityConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = IdentityConfig::load();
    println!("Starting Identity Service on {}:{} [{}]", config.host, config.port, config.env);

    // 1. Establish direct database connection
    // let db = Database::connect(&config.database_url).await?;
    // println!("Database connected: {}", config.database_url);

    // 2. Initialize the Repository
    // let user_repo = UserRepoImpl { db };

    // 3. We can now use user_repo directly (e.g. user_repo.find_by_pan(...).await)

    println!("Identity Service is ready!");
    Ok(())
}
