use database::sea_orm::Database;
use database::repo_impls::user::UserRepoImpl;
use domain::repository::user::UserRepository;

mod dto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Identity Service...");

    // 1. Establish direct database connection
    // Let's use a dummy DB URL for now if it's not set in env
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://user:pass@localhost:5432/virat".to_string());
    
    // In a real scenario, this connects directly without HTTP overhead
    // let db = Database::connect(&db_url).await?;
    println!("Database connected (simulation): {}", db_url);
    
    // 2. Initialize the Repository
    // let user_repo = UserRepoImpl { db };
    
    // 3. We can now use user_repo directly (e.g. user_repo.find_by_pan(...).await)
    
    println!("Identity Service is ready!");
    Ok(())
}
