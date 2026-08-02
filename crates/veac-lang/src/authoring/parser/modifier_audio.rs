use crate::authoring::{
    AudioCrossfadeDecl, AudioFadeCurveDecl, AudioModifierDecl, Identifier, PitchPolicyDecl,
    SemanticBlock, SemanticEntry, SemanticValue, Spanned,
};
use std::collections::HashSet;

use super::{audio_processor, parameter, semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    id: Identifier,
    mut body: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<AudioModifierDecl> {
    let gain = scalar(parser, &mut body, "gain")?;
    let pan = scalar(parser, &mut body, "pan")?;
    let muted = boolean(parser, &mut body, "muted")?;
    let normalize = boolean(parser, &mut body, "normalize")?;
    let pitch = match semantic::take(parser, &mut body, "pitch") {
        Some(entry) => Some(enum_value(
            parser,
            &entry,
            "pitch policy",
            |value| match value {
                "preserve" => Some(PitchPolicyDecl::Preserve),
                "follow-speed" => Some(PitchPolicyDecl::FollowSpeed),
                _ => None,
            },
        )?),
        None => None,
    };
    let processor_entries = take_all(&mut body, "processor");
    let processors = processor_entries
        .iter()
        .filter_map(|entry| audio_processor::parse(parser, entry))
        .collect::<Vec<_>>();
    duplicate_processor_ids(parser, &processors);
    let crossfade = match semantic::take(parser, &mut body, "crossfade") {
        Some(entry) => Some(crossfade(parser, &entry)?),
        None => None,
    };
    semantic::finish(parser, body, "audio modifier");
    Some(AudioModifierDecl {
        id,
        gain,
        pan,
        muted,
        normalize,
        pitch,
        processors,
        crossfade,
        span,
    })
}

fn duplicate_processor_ids(
    parser: &mut Parser,
    processors: &[crate::authoring::AudioProcessorDecl],
) {
    let mut seen = HashSet::new();
    for processor in processors {
        if !seen.insert(processor.id.value.as_str()) {
            parser.error(
                "AUTHORING_DUPLICATE_ID",
                format!("duplicate audio processor id '{}'", processor.id.value),
                processor.id.span,
            );
        }
    }
}

fn scalar(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::ParameterDecl<crate::authoring::NumberLiteral>>> {
    match semantic::take(parser, body, name) {
        Some(entry) => parameter::scalar(parser, &entry).map(Some),
        None => Some(None),
    }
}

fn boolean(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &'static str,
) -> Option<Option<Spanned<bool>>> {
    match semantic::take(parser, body, name) {
        Some(entry) => semantic::boolean(parser, &entry, name).map(Some),
        None => Some(None),
    }
}

fn crossfade(parser: &mut Parser, entry: &SemanticEntry) -> Option<AudioCrossfadeDecl> {
    let mut block = semantic::nested(parser, entry, "audio crossfade")?;
    let fade_in = required_number(parser, &mut block, "fade-in")?;
    let fade_out = required_number(parser, &mut block, "fade-out")?;
    let curve_entry = semantic::required(parser, &mut block, "curve", "audio crossfade")?;
    let curve = enum_value(
        parser,
        &curve_entry,
        "crossfade curve",
        |value| match value {
            "linear" => Some(AudioFadeCurveDecl::Linear),
            "equal-power" => Some(AudioFadeCurveDecl::EqualPower),
            "exponential" => Some(AudioFadeCurveDecl::Exponential),
            _ => None,
        },
    )?;
    semantic::finish(parser, block, "audio crossfade");
    Some(AudioCrossfadeDecl {
        fade_in,
        fade_out,
        curve,
        span: entry.span,
    })
}

fn required_number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    let entry = semantic::required(parser, block, name, "audio crossfade")?;
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

fn enum_value<T>(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Option<Spanned<T>> {
    let [SemanticValue::Identifier(value)] = entry.values.as_slice() else {
        semantic::shape(parser, entry, format!("{context} expects one value"));
        return None;
    };
    parse(&value.value)
        .map(|parsed| Spanned {
            value: parsed,
            span: value.span,
        })
        .or_else(|| {
            parser.error(
                "AUTHORING_UNKNOWN_ENUM_VALUE",
                format!("unknown {context} '{}'", value.value),
                value.span,
            );
            None
        })
}
