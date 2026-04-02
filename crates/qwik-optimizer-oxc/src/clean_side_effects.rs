//! Stage 11 post-transform DCE -- Treeshaker (CleanMarker + CleanSideEffects).
//!
//! After `QwikTransform` (Stage 10), the root module may contain
//! transform-introduced bare call/new expressions (e.g., `componentQrl(...)`)
//! that should be dropped for client-side builds.
//!
//! ## Algorithm
//!
//! 1. `CleanMarker::mark_module` -- First pass (before transform).
//!    Records `span.start` of every top-level `ExpressionStatement` whose
//!    expression is a `CallExpression` or `NewExpression`.
//!
//! 2. `CleanSideEffects::clean_module` -- Second pass (after transform).
//!    Retains only top-level expression-statement call/new expressions whose
//!    `span.start` is in the marker set.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use oxc::ast::ast::{Expression, Program, Statement};

// ---------------------------------------------------------------------------
// Treeshaker -- entry point
// ---------------------------------------------------------------------------

/// Combines `CleanMarker` and `CleanSideEffects` with a shared span set.
pub(crate) struct Treeshaker {
    pub marker: CleanMarker,
    pub cleaner: CleanSideEffects,
}

impl Treeshaker {
    pub(crate) fn new() -> Self {
        let set: Rc<RefCell<HashSet<u32>>> = Rc::new(RefCell::new(HashSet::new()));
        Treeshaker {
            marker: CleanMarker { spans: Rc::clone(&set) },
            cleaner: CleanSideEffects { spans: set, did_drop: false },
        }
    }
}

// ---------------------------------------------------------------------------
// CleanMarker -- pre-transform span recorder
// ---------------------------------------------------------------------------

pub(crate) struct CleanMarker {
    spans: Rc<RefCell<HashSet<u32>>>,
}

impl CleanMarker {
    pub(crate) fn mark_module(&self, program: &Program<'_>) {
        let mut set = self.spans.borrow_mut();
        for stmt in &program.body {
            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                match &expr_stmt.expression {
                    Expression::CallExpression(call) => {
                        set.insert(call.span.start);
                    }
                    Expression::NewExpression(new_expr) => {
                        set.insert(new_expr.span.start);
                    }
                    _ => {}
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CleanSideEffects -- post-transform call/new expression dropper
// ---------------------------------------------------------------------------

pub(crate) struct CleanSideEffects {
    spans: Rc<RefCell<HashSet<u32>>>,
    pub(crate) did_drop: bool,
}

impl CleanSideEffects {
    pub(crate) fn clean_module(&mut self, program: &mut Program<'_>) {
        let set = self.spans.borrow();
        let before = program.body.len();

        let mut keep = Vec::with_capacity(program.body.len());
        for (i, stmt) in program.body.iter().enumerate() {
            let drop = if let Statement::ExpressionStatement(expr_stmt) = stmt {
                match &expr_stmt.expression {
                    Expression::CallExpression(call) => !set.contains(&call.span.start),
                    Expression::NewExpression(new_expr) => {
                        !set.contains(&new_expr.span.start)
                    }
                    _ => false,
                }
            } else {
                false
            };
            if !drop {
                keep.push(i);
            }
        }

        if keep.len() < before {
            let mut to_remove: Vec<usize> = (0..before).filter(|i| !keep.contains(i)).collect();
            to_remove.reverse();
            for idx in to_remove {
                program.body.remove(idx);
            }
            self.did_drop = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    fn parse_program<'a>(allocator: &'a Allocator, src: &str) -> Program<'a> {
        let src = allocator.alloc_str(src);
        let ret = Parser::new(allocator, src, SourceType::default()).parse();
        assert!(!ret.panicked, "parse failed");
        ret.program
    }

    #[test]
    fn test_marker_records_call_spans() {
        let alloc = Allocator::default();
        let src = r#"foo(); bar(); const x = 1;"#;
        let program = parse_program(&alloc, src);

        let ts = Treeshaker::new();
        ts.marker.mark_module(&program);
        let set = ts.marker.spans.borrow();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_cleaner_preserves_non_call_stmts() {
        let alloc = Allocator::default();
        let src = r#"const x = 1; let y = foo();"#;
        let mut program = parse_program(&alloc, src);
        let original_len = program.body.len();

        let mut ts = Treeshaker::new();
        ts.marker.mark_module(&program);
        ts.cleaner.clean_module(&mut program);

        assert_eq!(program.body.len(), original_len);
        assert!(!ts.cleaner.did_drop);
    }

    #[test]
    fn test_treeshaker_end_to_end_drops_synthesised() {
        let alloc = Allocator::default();

        let pre_src = r#"userCall();"#;
        let pre_program = parse_program(&alloc, pre_src);

        let mut ts = Treeshaker::new();
        ts.marker.mark_module(&pre_program);

        let post_src = r#"userCall(); transformCall();"#;
        let mut post_program = parse_program(&alloc, post_src);
        ts.cleaner.clean_module(&mut post_program);

        assert!(ts.cleaner.did_drop);
        assert_eq!(post_program.body.len(), 1);
    }
}
