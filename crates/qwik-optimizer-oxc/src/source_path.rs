//! Newtype wrapper for source file paths.
//!
//! `SourcePath` wraps a `&str` file path and provides domain-specific methods
//! for source type detection and output extension computation. This keeps
//! path-related logic out of `parse.rs` and leverages Rust's type system
//! to distinguish "arbitrary string" from "a path to a source file."

use std::path::{Path, PathBuf};

use oxc::span::SourceType;

/// Decomposed path data for a single input module.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PathData {
    /// Filename without extension (e.g., "index" for "src/routes/index.tsx").
    pub file_stem: String,

    /// Filename with extension (e.g., "index.tsx").
    pub file_name: String,

    /// Directory portion of the relative path (e.g., "src/routes").
    /// Empty PathBuf when the file is in the root.
    pub rel_dir: PathBuf,

    /// Absolute directory path = src_dir.join(rel_dir).
    pub abs_dir: PathBuf,
}

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

    /// Decompose this source path into its constituent parts relative to `src_dir`.
    ///
    /// The inner string is treated as a slash-separated path relative to `src_dir`,
    /// e.g. `"src/routes/index.tsx"`.
    ///
    /// Returns a `PathData` with:
    /// - `file_stem`: filename without extension
    /// - `file_name`: filename with extension
    /// - `rel_dir`:   parent directory (empty PathBuf when no parent)
    /// - `abs_dir`:   `src_dir.join(rel_dir)`
    pub fn path_data(&self, src_dir: &Path) -> Result<PathData, anyhow::Error> {
        let rel_path = Path::new(self.0);

        let file_name = rel_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("path has no filename: {}", self.0))?
            .to_string();

        let file_stem = rel_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("path has no file stem: {}", self.0))?
            .to_string();

        let rel_dir = match rel_path.parent() {
            Some(p) if p != Path::new("") => p.to_path_buf(),
            _ => PathBuf::new(),
        };

        let abs_dir = src_dir.join(&rel_dir);

        Ok(PathData {
            file_stem,
            file_name,
            rel_dir,
            abs_dir,
        })
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

    // ---- path_data tests ---------------------------------------------------

    #[test]
    fn test_path_data_nested() {
        let src_dir = Path::new("/project");
        let result = SourcePath("src/routes/index.tsx").path_data(src_dir).unwrap();
        assert_eq!(result.file_stem, "index");
        assert_eq!(result.file_name, "index.tsx");
        assert_eq!(result.rel_dir, PathBuf::from("src/routes"));
        assert_eq!(result.abs_dir, PathBuf::from("/project/src/routes"));
    }

    #[test]
    fn test_path_data_root_level() {
        let src_dir = Path::new("/project");
        let result = SourcePath("component.tsx").path_data(src_dir).unwrap();
        assert_eq!(result.file_stem, "component");
        assert_eq!(result.file_name, "component.tsx");
        assert_eq!(result.rel_dir, PathBuf::new());
        assert_eq!(result.abs_dir, PathBuf::from("/project"));
    }

    #[test]
    fn test_path_data_one_level() {
        let src_dir = Path::new("/app");
        let result = SourcePath("routes/index.ts").path_data(src_dir).unwrap();
        assert_eq!(result.file_stem, "index");
        assert_eq!(result.file_name, "index.ts");
        assert_eq!(result.rel_dir, PathBuf::from("routes"));
        assert_eq!(result.abs_dir, PathBuf::from("/app/routes"));
    }

    // ---- source_type tests ------------------------------------------------

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
