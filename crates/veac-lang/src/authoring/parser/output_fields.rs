use crate::authoring::{Identifier, OutputKeyword, SemanticBlock};

use super::semantic::{take, word};
use super::Parser;

mod recipe;
mod units;

pub(super) use recipe::*;
pub(super) use units::*;

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

pub(super) fn is_leaf_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value != "."
        && value != ".."
        && !value
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
}
