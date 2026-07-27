use crate::authoring::{
    Identifier, NumberLiteral, OutputKeyword, SemanticBlock, SemanticEntry, SemanticValue, Spanned,
};

use super::semantic::{boolean, nested, number, take, word};
use super::Parser;

pub(super) fn string(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &str,
) -> Option<Spanned<String>> {
    match entry.values.as_slice() {
        [SemanticValue::String(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            parser.error(
                "AUTHORING_FIELD_SHAPE",
                format!("{context} must be one string"),
                entry.span,
            );
            None
        }
    }
}

pub(super) fn take_block(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<SemanticBlock> {
    take(parser, block, name).and_then(|entry| nested(parser, &entry, name))
}

pub(super) fn take_word(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<Identifier> {
    take(parser, block, name).and_then(|entry| word(parser, &entry, name))
}

pub(super) fn field_enum<T: OutputKeyword>(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    output: &mut T,
) {
    if let Some(value) = take_word(parser, block, name) {
        match T::parse(&value.value) {
            Some(parsed) => *output = parsed,
            None => parser.error(
                "AUTHORING_OUTPUT_ENUM",
                format!("invalid {name} value '{}'", value.value),
                value.span,
            ),
        }
    }
}

pub(super) fn field_optional_enum<T: OutputKeyword>(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    output: &mut Option<T>,
) {
    let mut value = None;
    field_enum(parser, block, name, &mut value);
    if value.is_some() {
        *output = value;
    }
}

impl<T: OutputKeyword> OutputKeyword for Option<T> {
    fn parse(value: &str) -> Option<Self> {
        T::parse(value).map(Some)
    }

    fn token(&self) -> &'static str {
        self.as_ref().expect("present output enum").token()
    }
}

pub(super) fn take_u64(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<u64> {
    let literal = take(parser, block, name).and_then(|entry| number(parser, &entry, name))?;
    match literal.raw.parse() {
        Ok(value) => Some(value),
        Err(_) => {
            parser.error(
                "AUTHORING_OUTPUT_INTEGER",
                format!("{name} must be an unsigned integer"),
                literal.span,
            );
            None
        }
    }
}

macro_rules! integer_field {
    ($name:ident, $ty:ty) => {
        pub(super) fn $name(
            parser: &mut Parser,
            block: &mut SemanticBlock,
            field: &'static str,
            output: &mut $ty,
        ) {
            if let Some(value) = take_u64(parser, block, field) {
                match <$ty>::try_from(value) {
                    Ok(value) => *output = value,
                    Err(_) => parser.error(
                        "AUTHORING_OUTPUT_INTEGER",
                        format!("{field} is outside the supported range"),
                        block.span,
                    ),
                }
            }
        }
    };
}

integer_field!(field_u32, u32);
integer_field!(field_u8, u8);

pub(super) fn field_bool(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    output: &mut bool,
) {
    if let Some(value) = take(parser, block, name).and_then(|entry| boolean(parser, &entry, name)) {
        *output = value.value;
    }
}

pub(super) fn field_string(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    output: &mut Option<String>,
) {
    if let Some(value) = take(parser, block, name).and_then(|entry| string(parser, &entry, name)) {
        *output = Some(value.value);
    }
}

pub(super) fn take_number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<NumberLiteral> {
    take(parser, block, name).and_then(|entry| number(parser, &entry, name))
}

pub(super) fn is_leaf_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value != "."
        && value != ".."
        && !value
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
}
