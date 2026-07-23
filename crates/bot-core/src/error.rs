// Hapus baris `use std::fmt;` karena tidak terpakai
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

/// Error utama untuk semua komponen bot.
#[derive(Debug, Error)]
pub enum BotError {
    // --- I/O & Filesystem ---
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    // --- Serialization ---
    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] SerdeJsonError),

    #[error("YAML serialization error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    // --- Database ---
    #[error("Database error: {0}")]
    Database(String),

    #[error("Migration error: {0}")]
    Migration(String),

    // --- Exchange / API ---
    #[error("Exchange API error: {0}")]
    Exchange(String),

    #[error("Exchange API rate limited: retry after {retry_after:?}")]
    ExchangeRateLimited { retry_after: std::time::Duration },

    #[error("Exchange authentication failed: {0}")]
    ExchangeAuth(String),

    // --- Strategy & Trading ---
    #[error("Strategy error: {0}")]
    Strategy(String),

    #[error("Invalid signal: {0}")]
    InvalidSignal(String),

    #[error("Order error: {0}")]
    Order(String),

    #[error("Risk management error: {0}")]
    Risk(String),

    #[error("Position error: {0}")]
    Position(String),

    // --- Validation ---
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid parameter: {field} = {value}")]
    InvalidParameter { field: String, value: String },

    // --- Not Found ---
    #[error("Resource not found: {0}")]
    NotFound(String),

    // --- Time ---
    #[error("Time error: {0}")]
    Time(#[from] chrono::ParseError),

    // --- Internal / Unknown ---
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unknown error")]
    Unknown,
}

// Helper untuk memudahkan konversi dari String/&str ke BotError
impl From<&str> for BotError {
    fn from(s: &str) -> Self {
        BotError::Internal(s.to_string())
    }
}

impl From<String> for BotError {
    fn from(s: String) -> Self {
        BotError::Internal(s)
    }
}

// Hasil khusus untuk seluruh aplikasi
pub type BotResult<T> = Result<T, BotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BotError::Config("missing field".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing field");

        let err = BotError::InvalidParameter {
            field: "min_score".to_string(),
            value: "10".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid parameter: min_score = 10");
    }

    #[test]
    fn test_error_from_string() {
        let err: BotError = "something went wrong".into();
        assert!(matches!(err, BotError::Internal(s) if s == "something went wrong"));
    }
}
