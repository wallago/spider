//! Error handler implementation.

use thiserror::Error as ThisError;

/// Errors that can occur while running `canary`.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// ESP-IDF error, as its raw `esp_err_t` code.
    #[error("ESP error: code {0}")]
    ESP(i32),
    /// Spi Bus error, it is not set yet.
    #[error("SPI Bus is missing")]
    SpiBus,
    /// Spi communication error occur.
    #[error("SPI communication: {0}")]
    Spi(&'static str),
}

/// Type alias for the standard [`Result`] type.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use std::string::ToString;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_error() {
        let error = Error::ESP("display");
        assert_eq!("ESP error: `display`", error.to_string());
    }
}
