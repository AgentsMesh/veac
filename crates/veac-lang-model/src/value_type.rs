mod capabilities;
mod display;
mod model;
mod parse;
mod validation;

pub use crate::{FunctionEffect, PrimitiveType};
pub use model::{MapKeyType, ValueType, ValueTypeKind};
pub use parse::ValueTypeParseError;
pub use validation::{ValueTypeError, MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeConstructor {
    List,
    Map,
    Range,
    Function,
}

impl TypeConstructor {
    const fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Map => "map",
            Self::Range => "range",
            Self::Function => "fn",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "list" => Some(Self::List),
            "map" => Some(Self::Map),
            "range" => Some(Self::Range),
            "fn" => Some(Self::Function),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
