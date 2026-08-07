use crate::source_edit::SourceExpressionStep as Step;

use super::super::ast::{join_path, ExpressionKind, MatchPattern};
use super::Walker;

impl Walker {
    pub(super) fn control(&mut self, expression: &ExpressionKind, path: &mut Vec<Step>) {
        match expression {
            ExpressionKind::Match { scrutinee, arms } => {
                self.child(scrutinee, path, Step::MatchScrutinee);
                for arm in arms {
                    let pattern = match &arm.pattern {
                        MatchPattern::Variant { path, .. } => join_path(path),
                        MatchPattern::Wildcard { .. } => "_".to_owned(),
                    };
                    self.child(&arm.body, path, Step::MatchArm { pattern });
                }
            }
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.child(condition, path, Step::IfCondition);
                path.push(Step::IfThen);
                self.block(then_branch, path);
                path.pop();
                path.push(Step::IfElse);
                self.block(else_branch, path);
                path.pop();
            }
            _ => unreachable!("control traversal requires match or if"),
        }
    }
}
