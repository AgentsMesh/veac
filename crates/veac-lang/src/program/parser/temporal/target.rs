use super::{controls, segment, Diagnostic, Parser, TokenKind};
use crate::program::model::{TemporalApplyPath, TemporalItemPath, TemporalTarget};

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<TemporalTarget, Diagnostic> {
    if take(parser, controls::TEMPORAL_CLIP_KIND) {
        return item_path(parser).map(TemporalTarget::Clip);
    }
    if take(parser, controls::TEMPORAL_TEXT_KIND) {
        return item_path(parser).map(TemporalTarget::Text);
    }
    if take(parser, controls::TEMPORAL_CLIP_MASK_KIND) {
        return clip_mask(parser);
    }
    if take(parser, controls::TEMPORAL_CLIP_EFFECT_KIND) {
        return clip_effect(parser);
    }
    if take(parser, controls::TEMPORAL_APPLY_KIND) {
        return apply_path(parser).map(TemporalTarget::Apply);
    }
    if take(parser, controls::TEMPORAL_APPLY_MASK_KIND) {
        return apply_mask(parser);
    }
    if take(parser, controls::TEMPORAL_APPLY_EFFECT_KIND) {
        return apply_effect(parser);
    }
    Err(parser.error(
        "PROGRAM_TEMPORAL_TARGET_KIND",
        "expected one closed temporal target kind",
        parser.current().span,
    ))
}

fn clip_mask(parser: &mut Parser<'_>) -> Result<TemporalTarget, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let clip = item_segments_with_tail(parser)?;
    let mask_index = index(parser)?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(TemporalTarget::ClipMask { clip, mask_index })
}

fn clip_effect(parser: &mut Parser<'_>) -> Result<TemporalTarget, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let clip = item_segments_with_tail(parser)?;
    let effect = segment(parser, "effect key")?;
    let (parameter, _) = parser.local_id("effect parameter")?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(TemporalTarget::ClipEffect {
        clip,
        effect,
        parameter,
    })
}

fn apply_mask(parser: &mut Parser<'_>) -> Result<TemporalTarget, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let apply = apply_segments(parser)?;
    let mask_index = index(parser)?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(TemporalTarget::ApplyMask { apply, mask_index })
}

fn apply_effect(parser: &mut Parser<'_>) -> Result<TemporalTarget, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let apply = apply_segments(parser)?;
    let stage = segment(parser, "apply stage key")?;
    let effect = segment(parser, "effect key")?;
    let (parameter, _) = parser.local_id("effect parameter")?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(TemporalTarget::ApplyEffect {
        apply,
        stage,
        effect,
        parameter,
    })
}

fn item_path(parser: &mut Parser<'_>) -> Result<TemporalItemPath, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let path = item_segments(parser)?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(path)
}

fn item_segments(parser: &mut Parser<'_>) -> Result<TemporalItemPath, Diagnostic> {
    Ok(TemporalItemPath {
        project: segment(parser, "project key")?,
        sequence: segment(parser, "sequence key")?,
        layer: segment(parser, "layer key")?,
        item: segment_or_last(parser, "item key")?,
    })
}

fn item_segments_with_tail(parser: &mut Parser<'_>) -> Result<TemporalItemPath, Diagnostic> {
    Ok(TemporalItemPath {
        project: segment(parser, "project key")?,
        sequence: segment(parser, "sequence key")?,
        layer: segment(parser, "layer key")?,
        item: segment(parser, "item key")?,
    })
}

fn apply_path(parser: &mut Parser<'_>) -> Result<TemporalApplyPath, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let path = apply_segments_last(parser)?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(path)
}

fn apply_segments(parser: &mut Parser<'_>) -> Result<TemporalApplyPath, Diagnostic> {
    Ok(TemporalApplyPath {
        project: segment(parser, "project key")?,
        sequence: segment(parser, "sequence key")?,
        apply: segment(parser, "apply key")?,
    })
}

fn apply_segments_last(parser: &mut Parser<'_>) -> Result<TemporalApplyPath, Diagnostic> {
    Ok(TemporalApplyPath {
        project: segment(parser, "project key")?,
        sequence: segment(parser, "sequence key")?,
        apply: segment_or_last(parser, "apply key")?,
    })
}

fn segment_or_last(parser: &mut Parser<'_>, label: &str) -> Result<String, Diagnostic> {
    parser.local_id(label).map(|(value, _)| value)
}

fn index(parser: &mut Parser<'_>) -> Result<u32, Diagnostic> {
    let token = parser.advance();
    let TokenKind::Number(value) = token.kind else {
        return Err(parser.error(
            "PROGRAM_TEMPORAL_INDEX",
            "expected a mask index",
            token.span,
        ));
    };
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(parser.error(
            "PROGRAM_TEMPORAL_INDEX",
            "mask index must be a non-negative integer",
            token.span,
        ));
    }
    value.parse().map_err(|_| {
        parser.error(
            "PROGRAM_TEMPORAL_INDEX",
            "mask index exceeds the supported range",
            token.span,
        )
    })
}

fn take(parser: &mut Parser<'_>, control: crate::vocabulary::ControlUse) -> bool {
    if parser.at_control(control) {
        parser.advance();
        true
    } else {
        false
    }
}
