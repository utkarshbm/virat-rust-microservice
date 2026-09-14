pub mod request_logger;
pub mod tracing_setup;
pub mod logger;
pub use tracing_setup::{init_tracing, LogGuards};
// pub use logger::Logger;

// Re-export core tracing items for convenient structured logging across all crates
pub use tracing::{debug, error, info, trace, warn, span, Level};

//re-export middlewares traits (transform + service)
pub use request_logger::{AuditLoggingMiddleware, AuditLoggingMiddlewareService, RequestMetadata};
