use crate::test_support::{sample_project, time};

use super::*;

#[test]
fn effects_require_a_compatible_authored_media_domain() {
    let mut audio_without_stream = sample_project();
    let clip = &mut audio_without_stream.project.sequences[0].tracks[0].clips[0];
    clip.audio = None;
    clip.effects[0].effect = Effect::neutral(EffectKind::AudioNormalize);
    assert_code(
        &validation_codes(&audio_without_stream),
        "EFFECT_MEDIA_TYPE",
    );

    let mut video_on_audio = sample_project();
    video_on_audio.project.sequences[0].tracks[0].kind = TrackKind::Audio;
    video_on_audio.project.sequences[0].tracks[0].clips[0].visual = None;
    assert_code(&validation_codes(&video_on_audio), "EFFECT_MEDIA_TYPE");
}

#[test]
fn visual_only_sources_reject_silently_ignored_audio_properties() {
    for source in [
        ClipSource::FreezeFrame {
            material_id: MaterialId::new("med_video").unwrap(),
            source_time: time(0),
        },
        ClipSource::Generated {
            generator: Generator::Transparent,
        },
    ] {
        let mut project = sample_project();
        let clip = &mut project.project.sequences[0].tracks[0].clips[0];
        clip.source = source;
        clip.source_mapping = None;
        assert_code(&validation_codes(&project), "AUDIO_SOURCE_TYPE");
    }
}
