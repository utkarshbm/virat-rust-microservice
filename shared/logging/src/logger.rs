#[derive(Debug, Clone, Copy)]
pub struct Logger {
    context: &'static str,
}

impl Logger {
    pub const fn new(context: &'static str) -> Self {
        Self { context }
    }

    pub fn log(&self, msg: &str) {
        tracing::info!(context = self.context, "{msg}");
    }

    pub fn warn(&self, msg: &str) {
        tracing::warn!(context = self.context, "{msg}");
    }

    pub fn error(&self, msg: &str) {
        tracing::error!(context = self.context, "{msg}");
    }

    pub fn debug(&self, msg: &str) {
        tracing::debug!(context = self.context, "{msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_methods() {
        let logger = Logger::new("TestContext");
        logger.log("This is an info log");
        logger.warn("This is a warning log");
        logger.error("This is an error log");
        logger.debug("This is a debug log");
    }
}
