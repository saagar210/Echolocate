/// Centralized error type for the Echolocate application
/// Provides structured error handling with codes, messages, and context

use serde::{Deserialize, Serialize};
use std::fmt;

/// Application error with structured information for frontend presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    /// Error code for frontend to handle (e.g., "SCAN_FAILED", "NETWORK_ERROR")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details for debugging
    pub details: Option<String>,
    /// ISO 8601 timestamp when error occurred
    pub timestamp: String,
}

impl AppError {
    /// Create a new AppError with code and message
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an AppError with additional context details
    pub fn with_details(code: &str, message: &str, details: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: Some(details.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create validation error
    pub fn validation(field: &str, reason: &str) -> Self {
        Self::new(
            "INVALID_INPUT",
            &format!("Validation failed for {}: {}", field, reason),
        )
    }

    /// Create network error
    pub fn network(reason: &str) -> Self {
        Self::new("NETWORK_ERROR", reason)
    }

    /// Create database error
    pub fn database(reason: &str) -> Self {
        Self::new("DATABASE_ERROR", reason)
    }

    /// Create scan error
    pub fn scan(reason: &str) -> Self {
        Self::new("SCAN_FAILED", reason)
    }

    /// Create command not found error
    pub fn command_not_found(command: &str) -> Self {
        Self::with_details(
            "COMMAND_NOT_FOUND",
            &format!("System command not available: {}", command),
            &format!("The command '{}' was not found in PATH. Please ensure it is installed.", command),
        )
    }

    /// Create parse error
    pub fn parse(source: &str, reason: &str) -> Self {
        Self::with_details(
            "PARSE_ERROR",
            &format!("Failed to parse {}", source),
            reason,
        )
    }

    /// Create permission denied error
    pub fn permission_denied(operation: &str) -> Self {
        Self::with_details(
            "PERMISSION_DENIED",
            &format!("Permission denied for: {}", operation),
            "This operation may require elevated privileges (sudo/admin).",
        )
    }

    /// Create timeout error
    pub fn timeout(operation: &str, seconds: u64) -> Self {
        Self::new(
            "TIMEOUT",
            &format!("{} timed out after {} seconds", operation, seconds),
        )
    }

    /// Create internal error (for unexpected conditions)
    pub fn internal(reason: &str) -> Self {
        Self::new("INTERNAL_ERROR", reason)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

// Conversion implementations

impl From<String> for AppError {
    fn from(e: String) -> Self {
        AppError::internal(&e)
    }
}

impl From<&str> for AppError {
    fn from(e: &str) -> Self {
        AppError::internal(e)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::database(&e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::new("FILE_NOT_FOUND", &format!("File not found: {}", e))
            }
            std::io::ErrorKind::PermissionDenied => {
                AppError::permission_denied(&e.to_string())
            }
            _ => AppError::new("IO_ERROR", &e.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::parse("JSON", &e.to_string())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(e: chrono::ParseError) -> Self {
        AppError::parse("timestamp", &e.to_string())
    }
}

// Tauri result type for command handlers
pub type TauriResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_creation() {
        let error = AppError::new("TEST_ERROR", "Test message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_none());
        assert!(!error.timestamp.is_empty());
    }

    #[test]
    fn test_app_error_with_details() {
        let error = AppError::with_details("TEST", "message", "extra context");
        assert_eq!(error.code, "TEST");
        assert_eq!(error.message, "message");
        assert_eq!(error.details, Some("extra context".to_string()));
    }

    #[test]
    fn test_app_error_display() {
        let error = AppError::new("CODE", "message");
        assert_eq!(error.to_string(), "[CODE] message");
    }

    #[test]
    fn test_app_error_validation() {
        let error = AppError::validation("device_name", "too long");
        assert_eq!(error.code, "INVALID_INPUT");
        assert!(error.message.contains("device_name"));
    }

    #[test]
    fn test_app_error_network() {
        let error = AppError::network("connection refused");
        assert_eq!(error.code, "NETWORK_ERROR");
    }

    #[test]
    fn test_app_error_database() {
        let error = AppError::database("table not found");
        assert_eq!(error.code, "DATABASE_ERROR");
    }

    #[test]
    fn test_app_error_command_not_found() {
        let error = AppError::command_not_found("arp");
        assert_eq!(error.code, "COMMAND_NOT_FOUND");
        assert!(error.message.contains("arp"));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_from_string() {
        let error: AppError = "test error".into();
        assert_eq!(error.code, "INTERNAL_ERROR");
    }
}
