use crate::env_vars::Environment;
use std::path::PathBuf;

/// Loads the appropriate `.env` file for a given service.
///
/// Resolution order:
/// 1. `services/{service_dir}/.env.{environment}` (e.g. `.env.development`)
/// 2. `services/{service_dir}/.env` (base fallback)
///
/// This function does NOT override variables that are already set in the
/// real environment. Real env vars always win over file values.
pub fn load_env_file(service_dir: &str) {
    let env = Environment::detect();
    let base_path = find_workspace_root();

    // Try environment-specific file first
    let env_specific = base_path
        .join("services")
        .join(service_dir)
        .join("envs")
        .join(format!(".env.{}", env.as_str()));

    if env_specific.exists() {
        println!(
            "[config] Loading env file: {}",
            env_specific.display()
        );
        dotenvy::from_path(&env_specific).ok();
    }

    // Then load base .env as fallback (won't override already-set vars)
    let env_base = base_path
        .join("services")
        .join(service_dir)
        .join("envs")
        .join(".env");

    if env_base.exists() {
        println!(
            "[config] Loading base env file: {}",
            env_base.display()
        );
        dotenvy::from_path(&env_base).ok();
    }

    println!("[config] Environment: {}", env);
}

/// Reads a required environment variable.
/// Panics with a clear error message if the variable is not set.
pub fn require_var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!(
            "\n[config] FATAL: Required environment variable '{}' is not set.\n\
             Hint: Check your .env files or set it in your shell.\n",
            name
        )
    })
}

/// Reads an optional environment variable, returning `default` if not set.
pub fn optional_var(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Walks up from the current directory to find the workspace root
/// (identified by the presence of a top-level `Cargo.toml` with `[workspace]`).
fn find_workspace_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("Failed to get current directory");
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return dir;
                }
            }
        }
        if !dir.pop() {
            // Fallback: use current directory if workspace root not found
            return std::env::current_dir().expect("Failed to get current directory");
        }
    }
}
