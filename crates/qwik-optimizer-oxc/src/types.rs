//! All public and internal type definitions for the qwik-optimizer-oxc crate.
//!
//! This is a pure data module with no logic -- only structs, enums, derives,
//! and serde attributes. Separating types into their own module prevents
//! circular dependencies since every other module can import from `types`
//! without importing logic.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public Types
// ---------------------------------------------------------------------------

/// Top-level configuration for transforming one or more modules.
///
/// SWC equivalent: TransformModulesOptions in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModulesOptions {
    /// Root directory for resolving relative paths.
    pub src_dir: String,

    /// Optional root directory override (used for monorepo setups).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_dir: Option<String>,

    /// List of input modules to transform.
    pub input: Vec<TransformModuleInput>,

    /// Whether to generate source maps.
    /// Default: true
    #[serde(default = "default_true")]
    pub source_maps: bool,

    /// Minification mode.
    /// Default: MinifyMode::Simplify
    #[serde(default)]
    pub minify: MinifyMode,

    /// Whether to strip TypeScript type annotations.
    /// Default: false
    #[serde(default)]
    pub transpile_ts: bool,

    /// Whether to transpile JSX to function calls.
    /// Default: false
    #[serde(default)]
    pub transpile_jsx: bool,

    /// Whether to preserve original filenames in output paths.
    /// Default: false
    #[serde(default)]
    pub preserve_filenames: bool,

    /// How to split extracted segments into output modules.
    /// Default: EntryStrategy::Segment
    #[serde(default)]
    pub entry_strategy: EntryStrategy,

    /// Whether to use explicit file extensions in import paths.
    /// Default: false
    #[serde(default)]
    pub explicit_extensions: bool,

    /// Output mode controlling which build target to emit for.
    /// Default: EmitMode::Lib
    #[serde(default)]
    pub mode: EmitMode,

    /// Optional scope prefix for segment names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Override the core module import path (default: "@qwik.dev/core").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_module: Option<String>,

    /// List of export names to strip from output.
    /// Used for server/client-specific builds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_exports: Option<Vec<String>>,

    /// List of ctx names to strip (e.g., strip all "useTask$" segments).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_ctx_name: Option<Vec<String>>,

    /// Whether to strip event handler registrations.
    /// Default: false
    #[serde(default)]
    pub strip_event_handlers: bool,

    /// List of ctx names to register (for plugin coordination).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reg_ctx_name: Option<Vec<String>>,

    /// Whether this build targets SSR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_server: Option<bool>,
}

fn default_true() -> bool {
    true
}

impl Default for TransformModulesOptions {
    fn default() -> Self {
        Self {
            src_dir: ".".to_string(),
            root_dir: None,
            input: vec![],
            source_maps: true,
            minify: MinifyMode::default(),
            transpile_ts: false,
            transpile_jsx: false,
            preserve_filenames: false,
            entry_strategy: EntryStrategy::default(),
            explicit_extensions: false,
            mode: EmitMode::default(),
            scope: None,
            core_module: None,
            strip_exports: None,
            strip_ctx_name: None,
            strip_event_handlers: false,
            reg_ctx_name: None,
            is_server: None,
        }
    }
}

/// A single input file to transform.
///
/// SWC equivalent: TransformModuleInput in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleInput {
    /// The source code content.
    pub code: String,

    /// The file path (relative to src_dir).
    pub path: String,

    /// Optional development path override (for HMR/dev mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_path: Option<String>,
}

/// Complete result of transforming one or more modules.
///
/// SWC equivalent: TransformOutput in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformOutput {
    /// All output modules: main modules + extracted segments.
    pub modules: Vec<TransformModule>,

    /// Diagnostics (errors, warnings) from transformation.
    pub diagnostics: Vec<Diagnostic>,

    /// Whether any input was TypeScript.
    pub is_type_script: bool,

    /// Whether any input contained JSX.
    pub is_jsx: bool,
}

impl Default for TransformOutput {
    fn default() -> Self {
        Self {
            modules: vec![],
            diagnostics: vec![],
            is_type_script: false,
            is_jsx: false,
        }
    }
}

impl TransformOutput {
    /// Merge another TransformOutput into this one.
    pub fn append(&mut self, other: &mut TransformOutput) {
        self.modules.append(&mut other.modules);
        self.diagnostics.append(&mut other.diagnostics);
        self.is_type_script = self.is_type_script || other.is_type_script;
        self.is_jsx = self.is_jsx || other.is_jsx;
    }
}

/// A single output module (either the transformed main module or an extracted segment).
///
/// SWC equivalent: TransformModule in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModule {
    /// Output file path (relative to src_dir).
    pub path: String,

    /// Whether this module is an entry point (segment modules are entry points).
    pub is_entry: bool,

    /// The generated JavaScript source code.
    pub code: String,

    /// Optional source map (JSON string, base64-encoded if inline).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,

    /// Segment metadata, present only for extracted segment modules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<SegmentAnalysis>,

    /// Original input file path (before transformation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_path: Option<String>,

    /// Sort order for deterministic output ordering.
    /// Main modules get order 0; segments get their extraction order.
    #[serde(default)]
    pub order: u64,
}

/// Metadata about an extracted segment (a lazy-loadable code fragment).
///
/// SWC equivalent: HookAnalysis/SegmentAnalysis in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentAnalysis {
    /// Source file this segment was extracted from.
    pub origin: String,

    /// Full segment name including hash (e.g., "renderHeader_zBbHWn4e8Cg").
    pub name: String,

    /// Entry point name, if this segment is a named entry.
    /// Always serialized (null when None) to match SWC golden format.
    pub entry: Option<String>,

    /// Human-readable display name (e.g., "test.tsx_renderHeader").
    pub display_name: String,

    /// The 11-character hash (e.g., "zBbHWn4e8Cg").
    pub hash: String,

    /// Canonical filename for the segment module (e.g., "test.tsx_renderHeader_zBbHWn4e8Cg").
    pub canonical_filename: String,

    /// Output path prefix (empty string if same directory).
    pub path: String,

    /// File extension of the output (e.g., "tsx", "ts", "js").
    pub extension: String,

    /// Parent segment name, if this segment is nested inside another.
    /// Always serialized (null when None) to match SWC golden format.
    pub parent: Option<String>,

    /// Context kind: whether this is an event handler or a function.
    pub ctx_kind: CtxKind,

    /// Context name: the $-suffixed function that created this segment
    /// (e.g., "$", "component$", "useTask$").
    pub ctx_name: String,

    /// Whether this segment captures variables from its enclosing scope.
    pub captures: bool,

    /// Names of captured variables (for inlinedQrl/qrl capture arrays).
    /// Only present when captures is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_names: Option<Vec<String>>,

    /// Source location as [start_byte, end_byte] of the original $-call.
    /// Serializes as a JSON array [start, end] (Rust tuples serialize as arrays).
    pub loc: (u32, u32),

    /// Parameter names after props destructuring transformation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_names: Option<Vec<String>>,
}

/// Controls how extracted segments are output.
///
/// SWC equivalent: EntryStrategy in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum EntryStrategy {
    /// Each segment becomes a separate file with a lazy import.
    /// This is the default strategy.
    Segment,

    /// Segments stay in the same file with inlinedQrl wrappers.
    Inline,

    /// Segments are hoisted to the top of the module.
    Hoist,

    /// All segments go into a single output file.
    Single,

    /// Group segments by their parent component.
    Component,

    /// Automatically choose the best strategy based on usage patterns.
    Smart,

    /// Group segments by their hook type.
    Hook,
}

impl Default for EntryStrategy {
    fn default() -> Self {
        EntryStrategy::Segment
    }
}

/// Controls output minification.
///
/// SWC equivalent: MinifyMode in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MinifyMode {
    /// Apply simplification transforms (dead code elimination, constant folding).
    Simplify,

    /// No minification.
    None,
}

impl Default for MinifyMode {
    fn default() -> Self {
        MinifyMode::Simplify
    }
}

/// Controls the build target output mode.
///
/// SWC equivalent: EmitMode in types.ts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EmitMode {
    /// Library mode -- standard output.
    Lib,

    /// Production mode -- optimized output (uses short `s_{hash}` symbol names).
    Prod,

    /// Development mode -- includes debug info.
    Dev,

    /// Hot Module Replacement mode -- development mode with HMR-specific transforms.
    Hmr,

    /// Test mode -- for unit test environments.
    Test,
}

impl Default for EmitMode {
    fn default() -> Self {
        EmitMode::Lib
    }
}

/// The kind of context that created a segment.
///
/// In spec file JSON, this appears as "eventHandler", "function", or "jsxProp".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CtxKind {
    /// Event handler context (e.g., onClick$, onInput$).
    #[serde(rename = "eventHandler")]
    EventHandler,

    /// Function context (e.g., $, component$, useTask$).
    #[serde(rename = "function")]
    Function,

    /// JSX prop expression context (e.g., inline JSX prop expressions).
    #[serde(rename = "jsxProp")]
    JSXProp,
}

/// A diagnostic message from the transformation process.
///
/// SWC equivalent: Diagnostic in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Scope identifier matching SWC wire format (always "optimizer").
    pub scope: String,

    /// The diagnostic category.
    pub category: DiagnosticCategory,

    /// Machine-readable error code.
    pub code: Option<String>,

    /// File path where the diagnostic originated.
    pub file: String,

    /// Human-readable message.
    pub message: String,

    /// Optional source code highlights.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<Vec<SourceLocation>>,

    /// Optional fix suggestions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticCategory {
    /// A hard error that prevents successful transformation.
    Error,

    /// A warning that does not prevent transformation.
    Warning,

    /// An error originating from the source code (e.g., syntax error).
    SourceError,
}

/// A source location for diagnostic highlighting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// Starting byte offset.
    pub lo: u32,

    /// Ending byte offset.
    pub hi: u32,

    /// Starting line (1-indexed).
    pub start_line: u32,

    /// Starting column (0-indexed).
    pub start_col: u32,

    /// Ending line (1-indexed).
    pub end_line: u32,

    /// Ending column (0-indexed).
    pub end_col: u32,
}

// ---------------------------------------------------------------------------
// Internal Types (used by transform stages, not part of public API)
// ---------------------------------------------------------------------------

/// Intermediate segment representation recorded during the transform pass.
/// This gets converted to SegmentAnalysis + a segment Program during code_move.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SegmentData {
    /// The display name (e.g., "test.tsx_Header_component").
    pub display_name: String,

    /// The computed hash (e.g., "J4uyIhaBNR4").
    pub hash: String,

    /// The full segment name (e.g., "Header_component_J4uyIhaBNR4").
    pub name: String,

    /// The callee that created this segment (e.g., "$", "component$").
    pub ctx_name: String,

    /// Context kind (function or event handler).
    pub ctx_kind: CtxKind,

    /// Source file origin.
    pub origin: String,

    /// File extension.
    pub extension: String,

    /// Span of the original $-call expression.
    pub span: (u32, u32),

    /// Parent segment name, if nested.
    pub parent: Option<String>,

    /// Variables from the enclosing scope that are referenced inside this segment.
    /// Used by `SmartStrategy` to decide whether the segment is "pure" (no captures).
    pub scoped_idents: Vec<String>,

    /// Whether the segment body captures outer scope variables.
    pub captures: bool,

    /// Names of captured variables (for inlinedQrl's capture array).
    pub capture_names: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_kind_serialization() {
        let kind = CtxKind::Function;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#""function""#);

        let kind = CtxKind::EventHandler;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#""eventHandler""#);

        let kind = CtxKind::JSXProp;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, r#""jsxProp""#);
    }

    #[test]
    fn test_transform_output_default() {
        let output = TransformOutput::default();
        assert!(output.modules.is_empty());
        assert!(output.diagnostics.is_empty());
        assert!(!output.is_type_script);
        assert!(!output.is_jsx);
    }

    #[test]
    fn test_transform_output_append() {
        let mut output1 = TransformOutput {
            modules: vec![TransformModule {
                path: "a.js".to_string(),
                is_entry: false,
                code: "".to_string(),
                map: None,
                segment: None,
                orig_path: None,
                order: 0,
            }],
            diagnostics: vec![],
            is_type_script: false,
            is_jsx: false,
        };
        let mut output2 = TransformOutput {
            modules: vec![TransformModule {
                path: "b.js".to_string(),
                is_entry: true,
                code: "".to_string(),
                map: None,
                segment: None,
                orig_path: None,
                order: 1,
            }],
            diagnostics: vec![],
            is_type_script: true,
            is_jsx: false,
        };
        output1.append(&mut output2);
        assert_eq!(output1.modules.len(), 2);
        assert!(output1.is_type_script);
    }

    #[test]
    fn test_entry_strategy_default() {
        let strategy = EntryStrategy::default();
        assert!(matches!(strategy, EntryStrategy::Segment));
    }

    #[test]
    fn test_emit_mode_default() {
        let mode = EmitMode::default();
        assert!(matches!(mode, EmitMode::Lib));
    }

    #[test]
    fn test_minify_mode_default() {
        let mode = MinifyMode::default();
        assert!(matches!(mode, MinifyMode::Simplify));
    }

    #[test]
    fn test_entry_strategy_serde_roundtrip() {
        let strategy = EntryStrategy::Segment;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, r#"{"type":"segment"}"#);

        let back: EntryStrategy = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, EntryStrategy::Segment));
    }

    #[test]
    fn test_emit_mode_serde_roundtrip() {
        let mode = EmitMode::Prod;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""prod""#);

        let back: EmitMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, EmitMode::Prod);
    }

    #[test]
    fn test_transform_modules_options_default_serde() {
        let opts = TransformModulesOptions::default();
        let json = serde_json::to_value(&opts).unwrap();
        assert_eq!(json["srcDir"], ".");
        assert_eq!(json["sourceMaps"], true);
        assert_eq!(json["transpileTs"], false);
        assert_eq!(json["transpileJsx"], false);
    }
}
