use crate::source_edit::SourceExpressionStep as Step;

use super::super::ast::ExpressionKind;
use super::Walker;

impl Walker {
    pub(super) fn compound(&mut self, expression: &ExpressionKind, path: &mut Vec<Step>) {
        match expression {
            ExpressionKind::Call { callee, arguments } => {
                self.child(callee, path, Step::CallCallee);
                self.indexed(arguments, path, |ordinal| Step::CallArgument { ordinal });
            }
            ExpressionKind::FieldProject { receiver, .. } => {
                self.child(receiver, path, Step::FieldReceiver)
            }
            ExpressionKind::NominalConstruct { fields, .. } => {
                for field in fields {
                    self.child(
                        &field.value,
                        path,
                        Step::NominalField {
                            field: field.name.clone(),
                        },
                    );
                }
            }
            ExpressionKind::TemporalAttach(attachment) => {
                self.indexed(&attachment.arguments, path, |ordinal| {
                    Step::TemporalArgument { ordinal }
                });
                path.push(Step::TemporalBody);
                self.block(&attachment.body, path);
                path.pop();
            }
            ExpressionKind::List(values) => {
                self.indexed(values, path, |ordinal| Step::ListItem { ordinal })
            }
            ExpressionKind::Map(entries) => {
                for (ordinal, entry) in entries.iter().enumerate() {
                    let ordinal = u32::try_from(ordinal).expect("expression node limit fits u32");
                    self.child(&entry.key, path, Step::MapKey { ordinal });
                    self.child(&entry.value, path, Step::MapValue { ordinal });
                }
            }
            ExpressionKind::Tuple(values) => {
                self.indexed(values, path, |ordinal| Step::TupleItem { ordinal })
            }
            ExpressionKind::Match { .. } | ExpressionKind::If { .. } => {
                self.control(expression, path)
            }
            _ => unreachable!("simple expression handled before compound traversal"),
        }
    }

    fn indexed(
        &mut self,
        values: &[super::super::ast::Expression],
        path: &mut Vec<Step>,
        step: impl Fn(u32) -> Step,
    ) {
        for (ordinal, value) in values.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).expect("expression node limit fits u32");
            self.child(value, path, step(ordinal));
        }
    }
}
