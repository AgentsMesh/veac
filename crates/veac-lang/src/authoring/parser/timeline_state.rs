use crate::authoring::{
    AudioStateDecl, EditingStateDecl, Identifier, IsolationStateDecl, ItemStateDecl, LayerKind,
    PlacementModeDecl, PlaybackStateDecl, SemanticBlock, Spanned, TrackStateDecl,
};

use super::semantic::{finish, required, take, word};
use super::Parser;

pub(super) fn placement(
    parser: &mut Parser,
    value: Identifier,
) -> Option<Spanned<PlacementModeDecl>> {
    let parsed = match value.value.as_str() {
        "free" => PlacementModeDecl::Free,
        "magnetic" => PlacementModeDecl::Magnetic,
        other => {
            parser.error(
                "AUTHORING_PLACEMENT_VALUE",
                format!("unsupported placement mode '{other}'"),
                value.span,
            );
            return None;
        }
    };
    Some(Spanned {
        value: parsed,
        span: value.span,
    })
}

pub(super) fn track(
    parser: &mut Parser,
    mut block: SemanticBlock,
    kind: LayerKind,
) -> Option<TrackStateDecl> {
    let playback = optional(parser, &mut block, "playback", playback)?;
    let audio = optional(parser, &mut block, "audio", audio)?;
    let isolation = optional(parser, &mut block, "isolation", isolation)?;
    let editing = optional(parser, &mut block, "editing", editing)?;
    if let Some(audio) = &audio {
        if !matches!(kind, LayerKind::Video | LayerKind::Audio) {
            parser.error(
                "AUTHORING_TRACK_AUDIO_STATE",
                "only video and audio layers accept audio state".to_owned(),
                audio.span,
            );
        }
    }
    let span = block.span;
    finish(parser, block, "track state");
    if playback.is_none() && audio.is_none() && isolation.is_none() && editing.is_none() {
        parser.error(
            "AUTHORING_TRACK_STATE_EMPTY",
            "track state requires at least one state axis".to_owned(),
            span,
        );
        return None;
    }
    Some(TrackStateDecl {
        playback,
        audio,
        isolation,
        editing,
        span,
    })
}

pub(super) fn item(parser: &mut Parser, mut block: SemanticBlock) -> Option<ItemStateDecl> {
    let entry = required(parser, &mut block, "playback", "item state")?;
    let playback = parse_word(parser, &entry, "playback", playback)?;
    let span = block.span;
    finish(parser, block, "item state");
    Some(ItemStateDecl { playback, span })
}

fn optional<T>(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    parse: fn(&str) -> Option<T>,
) -> Option<Option<Spanned<T>>> {
    match take(parser, block, name) {
        Some(entry) => parse_word(parser, &entry, name, parse).map(Some),
        None => Some(None),
    }
}

fn parse_word<T>(
    parser: &mut Parser,
    entry: &crate::authoring::SemanticEntry,
    name: &'static str,
    parse: fn(&str) -> Option<T>,
) -> Option<Spanned<T>> {
    let value = word(parser, entry, name)?;
    parse(&value.value)
        .map(|parsed| Spanned {
            value: parsed,
            span: value.span,
        })
        .or_else(|| {
            parser.error(
                "AUTHORING_STATE_VALUE",
                format!("unsupported {name} state '{}'", value.value),
                value.span,
            );
            None
        })
}

fn playback(value: &str) -> Option<PlaybackStateDecl> {
    match value {
        "enabled" => Some(PlaybackStateDecl::Enabled),
        "disabled" => Some(PlaybackStateDecl::Disabled),
        _ => None,
    }
}

fn audio(value: &str) -> Option<AudioStateDecl> {
    match value {
        "audible" => Some(AudioStateDecl::Audible),
        "muted" => Some(AudioStateDecl::Muted),
        _ => None,
    }
}

fn isolation(value: &str) -> Option<IsolationStateDecl> {
    match value {
        "normal" => Some(IsolationStateDecl::Normal),
        "solo" => Some(IsolationStateDecl::Solo),
        _ => None,
    }
}

fn editing(value: &str) -> Option<EditingStateDecl> {
    match value {
        "editable" => Some(EditingStateDecl::Editable),
        "locked" => Some(EditingStateDecl::Locked),
        _ => None,
    }
}
