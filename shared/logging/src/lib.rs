pub mod request_logger;
pub mod tracing_setup;

pub use tracing_setup::{init_tracing, LogGuards};

// Re-export core tracing items for convenient structured logging across all crates
pub use tracing::{debug, error, info, trace, warn, span, Level};
