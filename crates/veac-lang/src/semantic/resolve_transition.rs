/// Resolution of transition declarations.
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{ErrorKind, VeacError};
use crate::ir::*;

use super::SemanticAnalyzer;

impl SemanticAnalyzer<'_> {
    pub(crate) fn resolve_transition(
        &self,
        decl: &TransitionDecl,
        variables: &HashMap<String, Expression>,
        fps: u32,
    ) -> Result<IrTransition, VeacError> {
        let mut kind = TransitionKind::Fade;
        let mut duration_sec = 1.0;

        for attr in &decl.attributes {
            let val = self.resolve_expression(&attr.value, variables)?;
            match attr.key.as_str() {
                "type" => {
                    if let Expression::StringLit(s) = val {
                        kind = TransitionKind::parse(s).ok_or_else(|| {
                            VeacError::new(
                                ErrorKind::InvalidValue,
                                format!("unknown transition type `{s}`"),
                                None,
                            )
                            .with_hint("supported: fade, fadeblack, fadewhite, dissolve, wipe-left, wipe-right, wipe-up, wipe-down, slide-left, slide-right, slide-up, slide-down, zoom-in, smooth-left, smooth-right, smooth-up, smooth-down, squeeze-h, squeeze-v, circlecrop, pixelize")
                        })?;
                    }
                }
                "duration" => duration_sec = self.expr_to_seconds(val, fps, "duration")?,
                _ => {}
            }
        }

        Ok(IrTransition { kind, duration_sec })
    }
}
