use std::fmt;

mod capabilities;
mod display;
mod effect;
mod model;
mod parse;
mod validation;

use super::value::PrimitiveType;
pub use effect::FunctionEffect;
pub use model::{MapKeyType, ValueType, ValueTypeKind};
pub use parse::ValueTypeParseError;
pub use validation::{ValueTypeError, MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH};

crate::define_syntax_tokens! {
    array
    pub(crate) enum TypeConstructor {
        List => "list",
        Map => "map",
        Range => "range",
        Function => "fn",
    }
}

impl PrimitiveType {
    pub(crate) fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Scalar | Self::Time | Self::Length | Self::Percent | Self::Angle
        )
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests;
