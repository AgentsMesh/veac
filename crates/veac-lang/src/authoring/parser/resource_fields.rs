use super::Parser;
use crate::authoring::{ResourceIdentity, SemanticBlock, SemanticEntry, SemanticValue, Spanned};

pub(super) fn required_string(
    parser: &mut Parser,
    body: &SemanticBlock,
    name: &'static str,
) -> Option<Spanned<String>> {
    let entries = entries(parser, body, name);
    let Some(entry) = entries.first() else {
        parser.error(
            "AUTHORING_REQUIRED_FIELD",
            format!("locator requires {name}"),
            body.span,
        );
        return None;
    };
    match entry.values.as_slice() {
        [SemanticValue::String(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            parser.error(
                "AUTHORING_LOCATOR_FIELD",
                format!("locator {name} must be one string"),
                entry.span,
            );
            None
        }
    }
}

pub(super) fn remote_identity(
    parser: &mut Parser,
    body: &SemanticBlock,
) -> Option<ResourceIdentity> {
    let entries = entries(parser, body, "identity");
    let Some(entry) = entries.first() else {
        parser.error(
            "AUTHORING_REQUIRED_FIELD",
            "remote locator requires identity".to_owned(),
            body.span,
        );
        return None;
    };
    match entry.values.as_slice() {
        [SemanticValue::Identifier(kind), SemanticValue::String(value)]
            if kind.value == "sha256" && entry.block.is_none() =>
        {
            Some(ResourceIdentity::Sha256(value.clone()))
        }
        _ => {
            parser.error(
                "AUTHORING_RESOURCE_IDENTITY",
                "remote identity must be `identity sha256 \"...\";`".to_owned(),
                entry.span,
            );
            None
        }
    }
}

fn entries<'a>(
    parser: &mut Parser,
    body: &'a SemanticBlock,
    name: &'static str,
) -> Vec<&'a SemanticEntry> {
    let values: Vec<_> = body
        .entries
        .iter()
        .filter(|entry| entry.name.value == name)
        .collect();
    if values.len() > 1 {
        parser.duplicate(name, values[1].span);
    }
    values
}
