use crate::program::expression::{ValueType, ValueTypeError};
use crate::program::DomainType;

use super::{TypeRef, TypeSyntax, TypeSyntaxError, TypeSyntaxKind};

pub(super) fn value(
    syntax: &TypeSyntax,
    names: &dyn Fn(&str) -> Option<TypeRef>,
) -> Result<ValueType, TypeSyntaxError> {
    match syntax.kind() {
        TypeSyntaxKind::Primitive(value) => Ok(ValueType::primitive(*value)),
        TypeSyntaxKind::Named(name) => DomainType::parse(name)
            .map(ValueType::domain)
            .or_else(|| names(name).map(ValueType::nominal))
            .ok_or_else(|| unknown(name, syntax)),
        TypeSyntaxKind::List(element) => construct(syntax, ValueType::list(value(element, names)?)),
        TypeSyntaxKind::Range(element) => {
            construct(element, ValueType::range(value(element, names)?))
        }
        TypeSyntaxKind::Map { key, value: item } => {
            construct(key, ValueType::map(value(key, names)?, value(item, names)?))
        }
        TypeSyntaxKind::Tuple(elements) => construct(
            syntax,
            ValueType::tuple(
                elements
                    .iter()
                    .map(|element| value(element, names))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        TypeSyntaxKind::Function {
            parameters,
            result,
            effect,
        } => construct(
            syntax,
            ValueType::function(
                parameters
                    .iter()
                    .map(|parameter| value(parameter, names))
                    .collect::<Result<Vec<_>, _>>()?,
                value(result, names)?,
                *effect,
            ),
        ),
    }
}

fn construct(
    syntax: &TypeSyntax,
    result: Result<ValueType, ValueTypeError>,
) -> Result<ValueType, TypeSyntaxError> {
    result.map_err(|error| {
        TypeSyntaxError::new(
            match error.code() {
                "VALUE_TYPE_DEPTH" => "PROGRAM_TYPE_DEPTH",
                "VALUE_TYPE_ARITY" => "PROGRAM_TYPE_ARITY",
                "VALUE_TYPE_TUPLE_ARITY" => "PROGRAM_TYPE_TUPLE_ARITY",
                "VALUE_TYPE_MAP_KEY" => "PROGRAM_TYPE_MAP_KEY",
                "VALUE_TYPE_RANGE_ELEMENT" => "PROGRAM_TYPE_RANGE_ELEMENT",
                _ => "PROGRAM_VALUE_TYPE",
            },
            error.message(),
            syntax.span(),
        )
    })
}

fn unknown(name: &str, syntax: &TypeSyntax) -> TypeSyntaxError {
    TypeSyntaxError::new(
        "PROGRAM_UNKNOWN_TYPE",
        format!("unknown type `{name}`"),
        syntax.span(),
    )
}
