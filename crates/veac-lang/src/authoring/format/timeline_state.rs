use crate::authoring::{
    AudioStateDecl, EditingStateDecl, IsolationStateDecl, ItemStateDecl, PlacementModeDecl,
    PlaybackStateDecl, TrackStateDecl,
};

use super::writer::Writer;

pub(super) fn placement(value: PlacementModeDecl) -> &'static str {
    match value {
        PlacementModeDecl::Free => "free",
        PlacementModeDecl::Magnetic => "magnetic",
    }
}

pub(super) fn track(writer: &mut Writer, value: &TrackStateDecl) {
    writer.block("state", |writer| {
        if let Some(value) = &value.playback {
            writer.line(format!("playback {};", playback(value.value)));
        }
        if let Some(value) = &value.audio {
            writer.line(format!(
                "audio {};",
                match value.value {
                    AudioStateDecl::Audible => "audible",
                    AudioStateDecl::Muted => "muted",
                }
            ));
        }
        if let Some(value) = &value.isolation {
            writer.line(format!(
                "isolation {};",
                match value.value {
                    IsolationStateDecl::Normal => "normal",
                    IsolationStateDecl::Solo => "solo",
                }
            ));
        }
        if let Some(value) = &value.editing {
            writer.line(format!(
                "editing {};",
                match value.value {
                    EditingStateDecl::Editable => "editable",
                    EditingStateDecl::Locked => "locked",
                }
            ));
        }
    });
}

pub(super) fn item(writer: &mut Writer, value: &ItemStateDecl) {
    writer.block("state", |writer| {
        writer.line(format!("playback {};", playback(value.playback.value)));
    });
}

fn playback(value: PlaybackStateDecl) -> &'static str {
    match value {
        PlaybackStateDecl::Enabled => "enabled",
        PlaybackStateDecl::Disabled => "disabled",
    }
}
