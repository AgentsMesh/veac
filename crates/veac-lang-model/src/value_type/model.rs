use std::sync::Arc;

use super::{FunctionEffect, PrimitiveType};
use crate::{DomainType, TypeRef, TypeRegistry};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueType(pub(super) Repr);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Repr {
    Primitive(PrimitiveType),
    Domain(DomainType),
    Nominal(TypeRef),
    List(Arc<ValueType>),
    Range(Arc<ValueType>),
    Map {
        key: MapKeyType,
        value: Arc<ValueType>,
    },
    Tuple(Arc<[ValueType]>),
    Function {
        parameters: Arc<[ValueType]>,
        result: Arc<ValueType>,
        effect: FunctionEffect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapKeyType {
    Text,
    Identifier,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueTypeKind<'a> {
    Primitive(PrimitiveType),
    Domain(DomainType),
    Nominal(&'a TypeRef),
    List(&'a ValueType),
    Range(&'a ValueType),
    Map {
        key: MapKeyType,
        value: &'a ValueType,
    },
    Tuple(&'a [ValueType]),
    Function {
        parameters: &'a [ValueType],
        result: &'a ValueType,
        effect: FunctionEffect,
    },
}

impl ValueType {
    pub const fn primitive(value: PrimitiveType) -> Self {
        Self(Repr::Primitive(value))
    }

    pub const fn domain(value: DomainType) -> Self {
        Self(Repr::Domain(value))
    }

    pub fn nominal(value: TypeRef) -> Self {
        Self(Repr::Nominal(value))
    }

    pub fn kind(&self) -> ValueTypeKind<'_> {
        match &self.0 {
            Repr::Primitive(value) => ValueTypeKind::Primitive(*value),
            Repr::Domain(value) => ValueTypeKind::Domain(*value),
            Repr::Nominal(value) => ValueTypeKind::Nominal(value),
            Repr::List(value) => ValueTypeKind::List(value),
            Repr::Range(value) => ValueTypeKind::Range(value),
            Repr::Map { key, value } => ValueTypeKind::Map { key: *key, value },
            Repr::Tuple(values) => ValueTypeKind::Tuple(values),
            Repr::Function {
                parameters,
                result,
                effect,
            } => ValueTypeKind::Function {
                parameters,
                result,
                effect: *effect,
            },
        }
    }

    pub fn as_primitive(&self) -> Option<PrimitiveType> {
        match self.0 {
            Repr::Primitive(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_domain(&self) -> Option<DomainType> {
        match self.0 {
            Repr::Domain(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        self.as_primitive().is_some_and(PrimitiveType::is_numeric)
    }

    #[doc(hidden)]
    pub fn is_direct_function(&self) -> bool {
        matches!(self.kind(), ValueTypeKind::Function { .. })
    }

    pub fn contains_function_in(&self, registry: &TypeRegistry) -> Option<bool> {
        match self.kind() {
            ValueTypeKind::Function { .. } => Some(true),
            ValueTypeKind::Domain(_) => Some(false),
            ValueTypeKind::Nominal(value) => registry.contains_function(value.id()),
            ValueTypeKind::List(value)
            | ValueTypeKind::Range(value)
            | ValueTypeKind::Map { value, .. } => value.contains_function_in(registry),
            ValueTypeKind::Tuple(values) => values.iter().try_fold(false, |found, value| {
                value
                    .contains_function_in(registry)
                    .map(|value| found || value)
            }),
            ValueTypeKind::Primitive(_) => Some(false),
        }
    }

    pub fn depth(&self) -> usize {
        match self.kind() {
            ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) | ValueTypeKind::Nominal(_) => 1,
            ValueTypeKind::List(value)
            | ValueTypeKind::Range(value)
            | ValueTypeKind::Map { value, .. } => 1 + value.depth(),
            ValueTypeKind::Tuple(values) => 1 + values.iter().map(Self::depth).max().unwrap_or(0),
            ValueTypeKind::Function {
                parameters, result, ..
            } => {
                1 + parameters
                    .iter()
                    .chain(std::iter::once(result))
                    .map(Self::depth)
                    .max()
                    .unwrap_or(0)
            }
        }
    }
}

impl From<PrimitiveType> for ValueType {
    fn from(value: PrimitiveType) -> Self {
        Self::primitive(value)
    }
}

impl MapKeyType {
    pub const fn primitive(self) -> PrimitiveType {
        match self {
            Self::Text => PrimitiveType::Text,
            Self::Identifier => PrimitiveType::Identifier,
        }
    }
}
