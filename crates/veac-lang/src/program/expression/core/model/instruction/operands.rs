use super::CoreInstructionKind;
use crate::program::expression::ValueId;

impl CoreInstructionKind {
    pub(crate) fn operands(&self) -> std::vec::IntoIter<ValueId> {
        let mut values = match self {
            Self::Unary { operand, .. }
            | Self::LocalInit { value: operand, .. }
            | Self::LocalSet { value: operand, .. }
            | Self::TemporalProject { value: operand, .. }
            | Self::StructProject {
                structure: operand, ..
            } => {
                vec![*operand]
            }
            Self::Arithmetic { left, right, .. }
            | Self::Compare { left, right, .. }
            | Self::Equal { left, right, .. } => vec![*left, *right],
            Self::MapKey { builder, key, .. } => vec![*builder, *key],
            Self::MapValue { pending, value } => vec![*pending, *value],
            Self::MapFinish { builder } => vec![*builder],
            Self::Range { start, end, .. } => vec![*start, *end],
            Self::Invoke { callee, .. } => vec![*callee],
            Self::TemporalAttach { owner, .. } => vec![*owner],
            Self::Collection {
                iterable,
                initial,
                callable,
                ..
            } => {
                let mut values = vec![*iterable];
                values.extend(initial);
                values.push(*callable);
                values
            }
            _ => Vec::new(),
        };
        match self {
            Self::Call { arguments, .. } => values.extend(arguments),
            Self::TemporalAttach {
                selectors,
                animation,
                ..
            } => {
                values.extend(selectors);
                values.push(*animation);
            }
            Self::DomainConstruct { operands, .. }
            | Self::GraphEmit { operands, .. }
            | Self::TemporalCompose { operands, .. } => values.extend(operands),
            Self::Closure { captures, .. } => values.extend(captures),
            Self::Invoke { arguments, .. } => values.extend(arguments),
            Self::List { elements }
            | Self::Tuple { elements }
            | Self::StructConstruct {
                fields: elements, ..
            }
            | Self::EnumConstruct {
                fields: elements, ..
            } => values.extend(elements),
            Self::Range {
                step: Some(step), ..
            } => values.push(*step),
            _ => {}
        }
        values.into_iter()
    }
}
