use std::fmt;

/// Represents the 4 deployment environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Testing,
    Uat,
    Production,
}

impl Environment {
    /// Detects the current environment from the `RUN_ENV` variable.
    /// Defaults to `Development` if unset or unrecognized.
    pub fn detect() -> Self {
        match std::env::var("RUN_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "development" | "dev" => Self::Development,
            "testing" | "test" => Self::Testing,
            "uat" => Self::Uat,
            "production" | "prod" => Self::Production,
            other => {
                eprintln!(
                    "[config] WARNING: Unknown RUN_ENV '{}', defaulting to Development",
                    other
                );
                Self::Development
            }
        }
    }

    /// Returns the lowercase string used for file lookups (e.g. `.env.development`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Testing => "testing",
            Self::Uat => "uat",
            Self::Production => "production",
        }
    }

    /// Returns `true` if this is the production environment.
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
