use std::sync::Arc;

use super::model::{MapKeyType, Repr, ValueType};
use super::FunctionEffect;
use super::PrimitiveType;

pub const MAX_VALUE_TYPE_DEPTH: usize = 32;
pub const MAX_VALUE_TYPE_ARITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueTypeError {
    code: &'static str,
    message: String,
}

impl ValueType {
    pub fn list(element: Self) -> Result<Self, ValueTypeError> {
        bounded(Repr::List(Arc::new(element)))
    }

    pub fn range(element: Self) -> Result<Self, ValueTypeError> {
        if element.as_primitive() != Some(PrimitiveType::Integer) {
            return Err(ValueTypeError::new(
                "VALUE_TYPE_RANGE_ELEMENT",
                "range element type must be int",
            ));
        }
        bounded(Repr::Range(Arc::new(element)))
    }

    pub fn map(key: Self, value: Self) -> Result<Self, ValueTypeError> {
        let key = MapKeyType::try_from(&key)?;
        bounded(Repr::Map {
            key,
            value: Arc::new(value),
        })
    }

    pub fn tuple(elements: Vec<Self>) -> Result<Self, ValueTypeError> {
        require_arity("tuple", elements.len(), 2)?;
        bounded(Repr::Tuple(elements.into()))
    }

    pub fn function(
        parameters: Vec<Self>,
        result: Self,
        effect: FunctionEffect,
    ) -> Result<Self, ValueTypeError> {
        require_arity("function type", parameters.len(), 0)?;
        bounded(Repr::Function {
            parameters: parameters.into(),
            result: Arc::new(result),
            effect,
        })
    }
}

impl TryFrom<&ValueType> for MapKeyType {
    type Error = ValueTypeError;

    fn try_from(value: &ValueType) -> Result<Self, Self::Error> {
        match value.as_primitive() {
            Some(PrimitiveType::Text) => Ok(Self::Text),
            Some(PrimitiveType::Identifier) => Ok(Self::Identifier),
            _ => Err(ValueTypeError::new(
                "VALUE_TYPE_MAP_KEY",
                "map key type must be text or identifier",
            )),
        }
    }
}

impl ValueTypeError {
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn bounded(repr: Repr) -> Result<ValueType, ValueTypeError> {
    let value = ValueType(repr);
    if value.depth() > MAX_VALUE_TYPE_DEPTH {
        Err(ValueTypeError::new(
            "VALUE_TYPE_DEPTH",
            format!("type exceeds the {MAX_VALUE_TYPE_DEPTH} level depth limit"),
        ))
    } else {
        Ok(value)
    }
}

fn require_arity(name: &str, actual: usize, minimum: usize) -> Result<(), ValueTypeError> {
    if actual < minimum {
        Err(ValueTypeError::new(
            "VALUE_TYPE_TUPLE_ARITY",
            "tuple type requires at least two elements",
        ))
    } else if actual > MAX_VALUE_TYPE_ARITY {
        Err(ValueTypeError::new(
            "VALUE_TYPE_ARITY",
            format!("{name} exceeds the {MAX_VALUE_TYPE_ARITY} element arity limit"),
        ))
    } else {
        Ok(())
    }
}
