use crate::authoring::{
    AudioProcessorDecl, AudioProcessorKindDecl, CompressorDecl, FilterDecl, GateDecl, LimiterDecl,
    LoudnessDecl, SemanticBlock, SemanticEntry, SemanticValue,
};

use super::{semantic, Parser};

mod eq;

pub(super) fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<AudioProcessorDecl> {
    let [SemanticValue::Identifier(kind), SemanticValue::Identifier(id)] = entry.values.as_slice()
    else {
        semantic::shape(
            parser,
            entry,
            "processor expects one kind, one local id, and a block".to_owned(),
        );
        return None;
    };
    let block = entry.block.clone()?;
    let parsed = match kind.value.as_str() {
        "eq" => eq::parse(parser, block)?,
        "high-pass" => AudioProcessorKindDecl::HighPass(filter(parser, block, entry.span)?),
        "low-pass" => AudioProcessorKindDecl::LowPass(filter(parser, block, entry.span)?),
        "compressor" => AudioProcessorKindDecl::Compressor(compressor(parser, block, entry.span)?),
        "limiter" => AudioProcessorKindDecl::Limiter(limiter(parser, block, entry.span)?),
        "gate" => AudioProcessorKindDecl::Gate(gate(parser, block, entry.span)?),
        "loudness" => AudioProcessorKindDecl::Loudness(loudness(parser, block, entry.span)?),
        _ => {
            parser.error(
                "AUTHORING_UNKNOWN_AUDIO_PROCESSOR",
                format!("unknown audio processor '{}'", kind.value),
                kind.span,
            );
            return None;
        }
    };
    Some(AudioProcessorDecl {
        id: id.clone(),
        kind: parsed,
        span: entry.span,
    })
}

fn filter(
    parser: &mut Parser,
    mut block: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<FilterDecl> {
    let frequency = number(parser, &mut block, "frequency", "filter")?;
    let q = number(parser, &mut block, "q", "filter")?;
    let poles = number(parser, &mut block, "poles", "filter")?;
    semantic::finish(parser, block, "filter");
    Some(FilterDecl {
        frequency,
        q,
        poles,
        span,
    })
}

fn compressor(
    parser: &mut Parser,
    mut block: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<CompressorDecl> {
    let value = CompressorDecl {
        threshold: number(parser, &mut block, "threshold", "compressor")?,
        ratio: number(parser, &mut block, "ratio", "compressor")?,
        attack: number(parser, &mut block, "attack", "compressor")?,
        release: number(parser, &mut block, "release", "compressor")?,
        knee: number(parser, &mut block, "knee", "compressor")?,
        makeup_gain: number(parser, &mut block, "makeup-gain", "compressor")?,
        mix: number(parser, &mut block, "mix", "compressor")?,
        span,
    };
    semantic::finish(parser, block, "compressor");
    Some(value)
}

fn limiter(
    parser: &mut Parser,
    mut block: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<LimiterDecl> {
    let value = LimiterDecl {
        ceiling: number(parser, &mut block, "ceiling", "limiter")?,
        attack: number(parser, &mut block, "attack", "limiter")?,
        release: number(parser, &mut block, "release", "limiter")?,
        span,
    };
    semantic::finish(parser, block, "limiter");
    Some(value)
}

fn gate(
    parser: &mut Parser,
    mut block: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<GateDecl> {
    let value = GateDecl {
        threshold: number(parser, &mut block, "threshold", "gate")?,
        ratio: number(parser, &mut block, "ratio", "gate")?,
        attack: number(parser, &mut block, "attack", "gate")?,
        release: number(parser, &mut block, "release", "gate")?,
        range: number(parser, &mut block, "range", "gate")?,
        span,
    };
    semantic::finish(parser, block, "gate");
    Some(value)
}

fn loudness(
    parser: &mut Parser,
    mut block: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<LoudnessDecl> {
    let value = LoudnessDecl {
        integrated: number(parser, &mut block, "integrated", "loudness")?,
        true_peak: number(parser, &mut block, "true-peak", "loudness")?,
        range: number(parser, &mut block, "range", "loudness")?,
        span,
    };
    semantic::finish(parser, block, "loudness");
    Some(value)
}

fn number(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    let entry = semantic::required(parser, block, name, context)?;
    semantic::number(parser, &entry, name)
}
