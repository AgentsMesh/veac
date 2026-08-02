use std::collections::HashSet;

use crate::authoring::{
    AudioProcessorKindDecl, EqBandDecl, SemanticBlock, SemanticEntry, SemanticValue,
};

use super::super::{semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    mut block: SemanticBlock,
) -> Option<AudioProcessorKindDecl> {
    let entries = take_all(&mut block, "band");
    let bands = entries
        .iter()
        .filter_map(|entry| band(parser, entry))
        .collect::<Vec<_>>();
    duplicate_ids(parser, &bands);
    semantic::finish(parser, block, "parametric EQ");
    Some(AudioProcessorKindDecl::ParametricEq { bands })
}

fn band(parser: &mut Parser, entry: &SemanticEntry) -> Option<EqBandDecl> {
    let [SemanticValue::Identifier(id)] = entry.values.as_slice() else {
        semantic::shape(
            parser,
            entry,
            "EQ band expects one local id and a block".to_owned(),
        );
        return None;
    };
    let mut block = entry.block.clone()?;
    let frequency = number(parser, &mut block, "frequency")?;
    let gain = number(parser, &mut block, "gain")?;
    let q = number(parser, &mut block, "q")?;
    semantic::finish(parser, block, "EQ band");
    Some(EqBandDecl {
        id: id.clone(),
        frequency,
        gain,
        q,
        span: entry.span,
    })
}

fn duplicate_ids(parser: &mut Parser, bands: &[EqBandDecl]) {
    let mut seen = HashSet::new();
    for band in bands {
        if !seen.insert(band.id.value.as_str()) {
            parser.error(
                "AUTHORING_DUPLICATE_ID",
                format!("duplicate EQ band id '{}'", band.id.value),
                band.id.span,
            );
        }
    }
}

fn number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    let entry = semantic::required(parser, block, name, "EQ band")?;
    semantic::number(parser, &entry, name)
}

fn take_all(block: &mut SemanticBlock, name: &str) -> Vec<SemanticEntry> {
    let (matching, rest): (Vec<_>, Vec<_>) = block
        .entries
        .drain(..)
        .partition(|entry| entry.name.value == name);
    block.entries = rest;
    matching
}
