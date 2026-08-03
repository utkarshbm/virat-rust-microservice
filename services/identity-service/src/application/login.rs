use std::sync::Arc;
use domain::repository::user::UserRepository;
use cache::redis_client::RedisCache;

pub async fn login(
    user_repo: Arc<dyn UserRepository>,
    cache: Arc<RedisCache>,
    pan: &str,
    password: &str,
) -> Result<String, String> {
    // 1. Fetch AuthUser directly using find_by_pan
    let user = user_repo.find_by_pan(pan).await
        .map_err(|_| "internal error".to_string())?
        .ok_or("invalid credentials".to_string())?;

    // 2. Verify password (TODO: Implement crypto)
    // if !crypto::hashing::verify(password, &user.password_hash) {
    //     return Err("invalid credentials".to_string());
    // }

    // 3. Issue JWT & save token in Redis (TODO: Implement crypto)
    // let token = crypto::signing::generate_jwt(user.uuid());
    let token = "temp_jwt_token".to_string(); // Placeholder
    cache.store_token(user.uuid(), &token, 3600).await
        .map_err(|e| e.to_string())?;

    user_repo.mark_login(user.uuid()).await
        .map_err(|_| "internal error".to_string())?;

    Ok(token)
}