use veac_ir::{Animatable, AudioFadeCurve, AudioProcessorKind, Effect, FillMode, SlotKind};

use super::support;

#[path = "audio_effect_template_v6/coverage.rs"]
mod coverage;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let image = image_resource(
        identifier("image"), resource_file("assets/image.png"),
        sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    let voice = audio_resource(
        identifier("voice"), resource_file("assets/voice.wav"),
        sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        stream_auto()
    );
    let card = item(
        identifier("card"), item_enabled(), during(0s, 4s),
        source_media(image), source_timing_native()
    ).with_effect(video_blur_effect(
        identifier("blur"), effect_enabled(effect_window_during(during(1s, 2s))),
        length_constant(5px)
    )).with_effect(video_color_adjust_effect(
        identifier("grade"), effect_disabled(effect_window_full()),
        scalar_constant(0.1), scalar_constant(1.2), scalar_constant(0.9)
    )).with_template(template_contract(
        slot_image(), fill_take_center(), "主图替换位",
        source_duration_any(), template_text_locked()
    ));
    let audio = item(
        identifier("voice"), item_enabled(), during(0s, 4s),
        source_media(voice), source_timing_native()
    ).with_audio(audio_style(
        scalar_constant(0.8), scalar_constant(-0.2),
        audio_playback(false, false, pitch_preserve()),
        [
            audio_parametric_eq(identifier("eq"), [
                parametric_eq_band(identifier("presence"), 3000.0, 2.0, 1.2)
            ]),
            audio_high_pass(identifier("rumble"), 80.0, 0.7, 2),
            audio_low_pass(identifier("air"), 18000.0, 0.7, 2),
            audio_compressor(identifier("compressor"),
                compressor_settings(-18.0, 3.0, 10.0, 120.0, 4.0, 2.0, 75%)),
            audio_limiter(identifier("limiter"), limiter_settings(-1.0, 5.0, 100.0)),
            audio_gate(identifier("gate"), gate_settings(-50.0, 2.0, 5.0, 80.0, -30.0)),
            audio_loudness(identifier("loudness"), loudness_target(-16.0, -1.0, 8.0))
        ],
        audio_crossfade_present(1s, 1s, audio_fade_equal_power())
    )).with_effect(audio_normalize_effect(
        identifier("normalize"), effect_enabled(effect_window_full()), -14.0
    ));
    let state = track_state(
        track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked()
    );
    let visuals = visual_layer(
        identifier("visual"), 0, placement_free(), state, track_routing_default()
    ).with_item(card);
    let audio_track = audio_layer(
        identifier("audio"), 1, placement_free(), state, track_routing_default()
    ).with_item(audio);
    let sequence = sequence(
        identifier("main"), "主时间线",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
    ).with_layer(visuals).with_layer(audio_track);
    project(identifier("demo"), project_settings(600))
        .with_resource(image).with_resource(voice)
        .with_sequence(sequence).entry(sequence)
}
"#;

#[test]
fn audio_effect_and_template_families_lower_exactly() {
    let envelope = support::envelope(SOURCE);
    let sequence = &envelope.project.sequences[0];
    let card = &sequence.tracks[0].clips[0];
    assert_eq!(card.effects.len(), 2);
    assert!(matches!(&card.effects[0].effect, Effect::VideoBlur { .. }));
    assert_eq!(card.effects[0].enable_range.unwrap().start.value, 600);
    assert!(!card.effects[1].enabled);
    assert!(matches!(
        &card.effects[1].effect,
        Effect::VideoColorAdjust { contrast, .. }
            if contrast == &Animatable::constant(1.2)
    ));
    let slot = card.replaceable.as_ref().unwrap();
    assert_eq!(
        (slot.kind, slot.fill),
        (SlotKind::Image, FillMode::TakeCenter)
    );
    assert!(!card.template_editable_text);

    let audio = sequence.tracks[1].clips[0].audio.as_ref().unwrap();
    assert_eq!(audio.processors.len(), 7);
    assert!(matches!(
        audio.processors[0].kind,
        AudioProcessorKind::ParametricEq { .. }
    ));
    assert!(matches!(
        audio.processors[6].kind,
        AudioProcessorKind::Loudness(_)
    ));
    assert_eq!(audio.crossfade.unwrap().curve, AudioFadeCurve::EqualPower);
    assert!(matches!(
        &sequence.tracks[1].clips[0].effects[0].effect,
        Effect::AudioNormalize { target_lufs: -14.0 }
    ));
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn keyed_audio_and_effect_identities_are_deterministic() {
    let first = support::envelope(SOURCE);
    let second = support::envelope(SOURCE);
    assert_eq!(first, second);
    let audio = first.project.sequences[0].tracks[1].clips[0]
        .audio
        .as_ref()
        .unwrap();
    assert!(audio
        .processors
        .iter()
        .all(|value| value.id.as_str().starts_with("aud_")));
}
