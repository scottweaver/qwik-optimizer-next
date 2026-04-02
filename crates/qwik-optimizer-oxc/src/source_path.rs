//! Newtype wrapper for source file paths.
//!
//! `SourcePath` wraps a `&str` file path and provides domain-specific methods
//! for source type detection and output extension computation. This keeps
//! path-related logic out of `parse.rs` and leverages Rust's type system
//! to distinguish "arbitrary string" from "a path to a source file."

use oxc::span::SourceType;

/// A borrowed reference to a source file path (e.g., `"src/routes/index.tsx"`).
///
/// Provides methods for detecting the OXC `SourceType` from the file extension
/// and computing the correct output extension given transpilation flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourcePath<'a>(pub &'a str);

impl<'a> SourcePath<'a> {
    /// Detect `SourceType` from the file extension.
    ///
    /// - `.tsx` -> TSX (TypeScript + JSX)
    /// - `.ts`  -> TypeScript with JSX enabled (Qwik allows JSX in .ts files)
    /// - `.jsx` -> JSX (JavaScript + JSX)
    /// - `.js` / `.mjs` / `.cjs` -> ESM module (JavaScript)
    /// - Default: ESM module
    pub fn source_type(&self) -> SourceType {
        let filename = self.0;
        if filename.ends_with(".tsx") {
            SourceType::tsx()
        } else if filename.ends_with(".ts") {
            // Qwik allows JSX in .ts files, so enable JSX
            SourceType::ts().with_jsx(true)
        } else if filename.ends_with(".jsx") {
            SourceType::jsx()
        } else if filename.ends_with(".js")
            || filename.ends_with(".mjs")
            || filename.ends_with(".cjs")
        {
            SourceType::mjs()
        } else {
            SourceType::mjs()
        }
    }

    /// Map (transpile_ts, transpile_jsx, input extension) to the correct output extension.
    ///
    /// Rules per SPEC.md:
    /// - `.tsx` + transpile_ts + transpile_jsx -> `"js"`
    /// - `.tsx` + transpile_ts + !transpile_jsx -> `"jsx"`
    /// - `.tsx` + !transpile_ts + transpile_jsx -> `"ts"`  (strip JSX, keep TS)
    /// - `.ts`  + transpile_ts  -> `"js"`
    /// - `.jsx` + transpile_jsx -> `"js"`
    /// - Everything else -> preserve original extension
    pub fn output_extension(&self, transpile_ts: bool, transpile_jsx: bool) -> &'static str {
        let ext = self.0.rsplit('.').next().unwrap_or("");
        match ext {
            "tsx" => match (transpile_ts, transpile_jsx) {
                (true, true) => "js",
                (true, false) => "jsx",
                (false, true) => "ts",
                (false, false) => "tsx",
            },
            "ts" => {
                if transpile_ts {
                    "js"
                } else {
                    "ts"
                }
            }
            "jsx" => {
                if transpile_jsx {
                    "js"
                } else {
                    "jsx"
                }
            }
            "js" => "js",
            "mjs" => "mjs",
            "cjs" => "cjs",
            _ => "js", // unknown: default to js
        }
    }

    /// Returns the inner path string.
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl<'a> From<&'a str> for SourcePath<'a> {
    fn from(s: &'a str) -> Self {
        SourcePath(s)
    }
}

impl std::fmt::Display for SourcePath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_tsx() {
        let p = SourcePath("app.tsx");
        assert!(p.source_type().is_typescript());
        assert!(p.source_type().is_jsx());
    }

    #[test]
    fn test_source_type_ts() {
        let p = SourcePath("app.ts");
        assert!(p.source_type().is_typescript());
        assert!(p.source_type().is_jsx()); // Qwik enables JSX for .ts
    }

    #[test]
    fn test_source_type_jsx() {
        let p = SourcePath("app.jsx");
        assert!(!p.source_type().is_typescript());
        assert!(p.source_type().is_jsx());
    }

    #[test]
    fn test_source_type_js_variants() {
        for ext in &["app.js", "app.mjs", "app.cjs"] {
            let p = SourcePath(ext);
            assert!(!p.source_type().is_typescript());
        }
    }

    #[test]
    fn test_source_type_unknown_defaults_to_mjs() {
        let p = SourcePath("app.txt");
        assert!(!p.source_type().is_typescript());
    }

    #[test]
    fn test_output_extension_tsx_both_transpile() {
        assert_eq!(SourcePath("test.tsx").output_extension(true, true), "js");
    }

    #[test]
    fn test_output_extension_tsx_ts_only() {
        assert_eq!(SourcePath("test.tsx").output_extension(true, false), "jsx");
    }

    #[test]
    fn test_output_extension_tsx_jsx_only() {
        assert_eq!(SourcePath("test.tsx").output_extension(false, true), "ts");
    }

    #[test]
    fn test_output_extension_tsx_no_transpile() {
        assert_eq!(SourcePath("test.tsx").output_extension(false, false), "tsx");
    }

    #[test]
    fn test_output_extension_ts_transpile() {
        assert_eq!(SourcePath("test.ts").output_extension(true, true), "js");
    }

    #[test]
    fn test_output_extension_ts_no_transpile() {
        assert_eq!(SourcePath("test.ts").output_extension(false, true), "ts");
    }

    #[test]
    fn test_output_extension_jsx_transpile() {
        assert_eq!(SourcePath("test.jsx").output_extension(false, true), "js");
    }

    #[test]
    fn test_output_extension_js_unchanged() {
        assert_eq!(SourcePath("test.js").output_extension(true, true), "js");
        assert_eq!(SourcePath("test.js").output_extension(false, false), "js");
    }
}
