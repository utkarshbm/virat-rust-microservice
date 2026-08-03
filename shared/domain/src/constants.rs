/// Standard HTTP header name for tracing requests across microservices.
pub const TRACE_HEADER_NAME: &str = "x-request-id";

/// Default pagination size for list queries across all services.
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Standard maximum pagination size across all services.
pub const MAX_PAGE_SIZE: u32 = 100;
