/// Maximum amount allowed for a single transaction (in smallest currency unit, e.g. paise/cents).
/// 100,000 INR = 10,000,000 paise
pub const MAX_TRANSACTION_AMOUNT: u64 = 10_000_000;

/// Expiry time for a pending payment session in minutes.
pub const PAYMENT_SESSION_EXPIRY_MINUTES: u64 = 15;

/// Prefix for idempotency keys stored in Redis.
pub const REDIS_IDEMPOTENCY_PREFIX: &str = "idempotency:payment:";
