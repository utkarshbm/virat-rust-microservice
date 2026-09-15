pub mod logger;
pub mod request_logger;
pub mod tracing_setup;
pub use tracing_setup::{LogGuards, init_tracing};
// pub use logger::Logger;

// Re-export core tracing items for convenient structured logging across all crates
pub use tracing::{Level, debug, error, info, span, trace, warn};

//re-export middlewares traits (transform + service)
pub use request_logger::{
    AuditLoggingMiddleware, AuditLoggingMiddlewareService, DecryptedBody, RequestMetadata,
};
