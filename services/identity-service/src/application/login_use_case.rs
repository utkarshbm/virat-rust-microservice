use std::sync::Arc;
use domain::repository::user_repository::UserRepository;
use cache::redis_client::RedisCache;

pub async fn login(
    user_repo: Arc<dyn UserRepository>,
    cache: Arc<RedisCache>,
    pan: &str,
    password: &str,
) -> Result<String, String> {
    // 1. Fetch AuthUser (which includes password_hash)
    let user = user_repo.find_auth_by_pan(pan).await
        .map_err(|_| "internal error".to_string())?
        .ok_or("invalid credentials".to_string())?;

    // 2. Verify password against the hash
    if !crypto::hashing::verify(password, &user.password_hash) {
        return Err("invalid credentials".to_string());
    }

    // 3. Generate JWT using the user.uuid() helper method
    let token = crypto::signing::generate_jwt(user.uuid());

    // 4. Cache in Redis & update database state
    cache.store_token(user.uuid(), &token, 3600).await
        .map_err(|e| e.to_string())?;

    user_repo.mark_login(user.uuid()).await
        .map_err(|_| "internal error".to_string())?;

    Ok(token)
}