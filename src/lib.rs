//! Multi-screen info panel (Over ESP32).

/// Error handler implementation.
pub mod error;

/// Main application.
pub mod app;

/// Command-line arguments parser.
pub mod args;

/// Config file.
pub mod config;

/// Helper functions.
pub mod help;

/// Common types that can be glob-imported for convenience.
pub mod prelude;

use prelude::*;

use crate::config::Config;

/// Runs `spider`.
///
/// # Errors
///
/// Returns an [`Error`] if the run fa
pub fn run(args: &Args) -> Result<()> {
    better_panic::install();
    let _config = Config::load(args.config.as_deref())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn run_succeeds() {
        assert_eq!(run(Args::default()).is_ok(), true);
    }
}
