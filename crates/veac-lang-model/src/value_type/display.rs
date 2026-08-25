use std::fmt;

use super::{TypeConstructor, ValueType, ValueTypeKind};

impl fmt::Display for ValueType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            ValueTypeKind::Primitive(value) => value.fmt(formatter),
            ValueTypeKind::Domain(value) => value.fmt(formatter),
            ValueTypeKind::Nominal(value) => value.fmt(formatter),
            ValueTypeKind::List(element) => {
                write!(formatter, "{}<{element}>", TypeConstructor::List.as_str())
            }
            ValueTypeKind::Range(element) => {
                write!(formatter, "{}<{element}>", TypeConstructor::Range.as_str())
            }
            ValueTypeKind::Map { key, value } => write!(
                formatter,
                "{}<{}, {value}>",
                TypeConstructor::Map.as_str(),
                key.primitive()
            ),
            ValueTypeKind::Tuple(elements) => {
                formatter.write_str("(")?;
                separated(formatter, elements)?;
                formatter.write_str(")")
            }
            ValueTypeKind::Function {
                parameters,
                result,
                effect,
            } => {
                write!(formatter, "{}(", TypeConstructor::Function.as_str())?;
                separated(formatter, parameters)?;
                write!(formatter, ") -> {result} effect {effect}")
            }
        }
    }
}

fn separated(formatter: &mut fmt::Formatter<'_>, values: &[ValueType]) -> fmt::Result {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            formatter.write_str(", ")?;
        }
        fmt::Display::fmt(value, formatter)?;
    }
    Ok(())
}
