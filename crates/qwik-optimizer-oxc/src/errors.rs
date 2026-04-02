//! Diagnostic types and error helpers.
//!
//! Helper functions for creating `Diagnostic` values with consistent formatting.
//! Centralizes error message templates so the rest of the crate can report errors
//! without constructing Diagnostic structs manually.

use crate::types::{Diagnostic, DiagnosticCategory};

/// Create a source error diagnostic (e.g., syntax error).
pub(crate) fn create_source_error(message: &str, file: &str) -> Diagnostic {
    Diagnostic {
        scope: "optimizer".to_string(),
        category: DiagnosticCategory::SourceError,
        code: None,
        file: file.to_string(),
        message: message.to_string(),
        highlights: None,
        suggestions: None,
    }
}

/// Create a general error diagnostic.
pub(crate) fn create_error(message: &str, file: &str) -> Diagnostic {
    Diagnostic {
        scope: "optimizer".to_string(),
        category: DiagnosticCategory::Error,
        code: None,
        file: file.to_string(),
        message: message.to_string(),
        highlights: None,
        suggestions: None,
    }
}

/// Create a warning diagnostic.
pub(crate) fn create_warning(message: &str, file: &str) -> Diagnostic {
    Diagnostic {
        scope: "optimizer".to_string(),
        category: DiagnosticCategory::Warning,
        code: None,
        file: file.to_string(),
        message: message.to_string(),
        highlights: None,
        suggestions: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_source_error() {
        let d = create_source_error("parse error", "test.tsx");
        assert_eq!(d.scope, "optimizer");
        assert_eq!(d.file, "test.tsx");
        assert_eq!(d.message, "parse error");
        assert!(matches!(d.category, DiagnosticCategory::SourceError));
    }

    #[test]
    fn test_create_error() {
        let d = create_error("transform error", "app.tsx");
        assert!(matches!(d.category, DiagnosticCategory::Error));
    }

    #[test]
    fn test_create_warning() {
        let d = create_warning("deprecated usage", "old.tsx");
        assert!(matches!(d.category, DiagnosticCategory::Warning));
    }
}
