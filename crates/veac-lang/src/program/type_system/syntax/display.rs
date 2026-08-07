use std::fmt;

use super::{TypeSyntax, TypeSyntaxKind};

impl fmt::Display for TypeSyntax {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            TypeSyntaxKind::Primitive(value) => value.fmt(formatter),
            TypeSyntaxKind::Named(value) => formatter.write_str(value),
            TypeSyntaxKind::List(value) => write!(formatter, "list<{value}>"),
            TypeSyntaxKind::Range(value) => write!(formatter, "range<{value}>"),
            TypeSyntaxKind::Map { key, value } => write!(formatter, "map<{key}, {value}>"),
            TypeSyntaxKind::Tuple(values) => {
                formatter.write_str("(")?;
                separated(formatter, values)?;
                formatter.write_str(")")
            }
            TypeSyntaxKind::Function {
                parameters,
                result,
                effect,
            } => {
                formatter.write_str("fn(")?;
                separated(formatter, parameters)?;
                write!(formatter, ") -> {result} effect {effect}")
            }
        }
    }
}

fn separated(formatter: &mut fmt::Formatter<'_>, values: &[TypeSyntax]) -> fmt::Result {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            formatter.write_str(", ")?;
        }
        write!(formatter, "{value}")?;
    }
    Ok(())
}
