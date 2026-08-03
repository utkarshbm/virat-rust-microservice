/// Maximum number of failed login attempts before locking out an account temporarily.
pub const MAX_LOGIN_ATTEMPTS: u8 = 5;

/// Number of minutes before an OTP expires.
pub const OTP_EXPIRY_MINUTES: u64 = 10;

/// Prefix for storing user sessions in Redis.
pub const REDIS_SESSION_PREFIX: &str = "session:";
