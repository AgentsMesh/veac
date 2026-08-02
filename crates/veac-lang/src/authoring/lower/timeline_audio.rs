use crate::authoring::{GeneratorDecl, ItemDecl, MappingDecl, SourceDecl};
use veac_ir::{Animatable, AudioProperties, PitchPolicy, TrackKind};

use super::context::Context;

pub(super) fn resolve(
    ctx: &mut Context,
    value: &ItemDecl,
    kind: TrackKind,
    authored: Option<AudioProperties>,
) -> Option<Option<AudioProperties>> {
    let source = default_audio(ctx, value, kind);
    match (source, authored) {
        (Some(_), Some(authored)) => Some(Some(authored)),
        (Some(default), None) => Some(Some(default)),
        (None, Some(_)) => {
            ctx.error(
                "AUTHORING_LOWER_AUDIO_SOURCE",
                "audio modifiers require an audio-producing source on a video or audio layer",
                value.span,
            );
            None
        }
        (None, None) => Some(None),
    }
}

fn default_audio(ctx: &Context, value: &ItemDecl, kind: TrackKind) -> Option<AudioProperties> {
    let supports_audio = match &value.source {
        SourceDecl::Media { resource, .. } => ctx.audio_resources.contains(&resource.id.value),
        SourceDecl::Sequence { sequence, .. } => ctx.audio_sequences.contains(&sequence.id.value),
        SourceDecl::Multicam { .. } => true,
        _ => false,
    } || matches!(
        value.source,
        SourceDecl::Generated {
            generator: GeneratorDecl::Silence { .. },
            ..
        }
    );
    let freeze = matches!(value.mapping, Some(MappingDecl::Freeze { .. }));
    (supports_audio && !freeze && matches!(kind, TrackKind::Video | TrackKind::Audio)).then(audio)
}

fn audio() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: Vec::new(),
        crossfade: None,
    }
}
