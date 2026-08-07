use veac_ir::{ApplyOperation, ApplyTarget, RelationKind, TransitionAlignment, TransitionKind};

use super::support;

#[path = "relation_apply_v6/coverage.rs"]
mod coverage;

const SOURCE: &str = r#"
fn audio_clip(key: identifier) -> Item {
    item(key, item_enabled(), during(0s, 5s),
        source_generated(generator_silence()), source_timing_native())
        .with_audio(audio_style(
            scalar_constant(1.0), scalar_constant(0.0),
            audio_playback(false, false, pitch_preserve()),
            [], audio_crossfade_none()
        ))
}

fn main(context: Context) -> Project {
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let outgoing = item(identifier("outgoing"), item_enabled(), during(0s, 3s),
        source_generated(generator_solid(#cc3344ff)), source_timing_native());
    let incoming = item(identifier("incoming"), item_enabled(), during(2s, 3s),
        source_generated(generator_solid(#3366ccff)), source_timing_native());
    let matte = item(identifier("matte"), item_enabled(), during(0s, 5s),
        source_generated(generator_solid(#ffffffff)), source_timing_native());
    let linked = item(identifier("linked"), item_enabled(), during(0s, 5s),
        source_generated(generator_solid(#22aa66ff)), source_timing_native());
    let music = audio_clip(identifier("music"));
    let voice = audio_clip(identifier("voice"));
    let video = video_layer(identifier("video"), 0, placement_free(), state,
        track_routing_default()).with_item(outgoing).with_item(incoming);
    let matte_track = visual_layer(identifier("matte"), 1, placement_free(), state,
        track_routing_default()).with_item(matte);
    let linked_track = visual_layer(identifier("linked"), 2, placement_free(), state,
        track_routing_default()).with_item(linked);
    let music_track = audio_layer(identifier("music"), 3, placement_free(), state,
        track_routing_default()).with_item(music);
    let voice_track = audio_layer(identifier("voice"), 4, placement_free(), state,
        track_routing_default()).with_item(voice);
    let color = color_pipeline(
        color_space(primaries_bt709(), transfer_srgb(), matrix_rgb(), range_full()),
        color_space(primaries_bt709(), transfer_linear(), matrix_rgb(), range_full()),
        color_space(primaries_bt709(), transfer_srgb(), matrix_rgb(), range_full()),
        [color_stage_basic(basic_color(0.2, 6500.0, 0.0, 0.0, 0.0, 0.0))]
    );
    let grade = apply(
        identifier("grade"), apply_enabled(), during(0s, 5s),
        apply_target_layer(video),
        [
            apply_color_stage(identifier("color"),
                apply_stage_enabled(apply_stage_window_full()), color),
            apply_effect_stage(identifier("blur"),
                apply_stage_disabled(apply_stage_window_during(during(1s, 2s))),
                video_blur_effect(identifier("blur-effect"),
                    effect_enabled(effect_window_full()), length_constant(3px)))
        ],
        apply_mix(percent_constant(85%), blend_overlay(), [])
    );
    let transition = relation_transition(
        identifier("transition"), outgoing, incoming, transition_dissolve(1s));
    let matte_relation = relation_matte_apply(
        identifier("matte"), matte, grade, matte_alpha(), false);
    let sidechain = relation_sidechain_track(
        identifier("sidechain"), music_track, voice,
        sidechain_settings(-24.0, 4.0, 10.0, 120.0, sidechain_window_full()));
    let group = relation_group(identifier("group"), [outgoing, matte]);
    let av = relation_av_link(identifier("av"), linked, [music]);
    let sequence = sequence(identifier("main"), "关系与调整层",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(video).with_layer(matte_track).with_layer(linked_track)
        .with_layer(music_track).with_layer(voice_track)
        .with_apply(grade)
        .with_relation(transition).with_relation(matte_relation)
        .with_relation(sidechain).with_relation(group).with_relation(av);
    project(identifier("demo"), project_settings(600))
        .with_sequence(sequence).entry(sequence)
}
"#;

#[test]
fn relation_and_apply_algebras_lower_to_centered_typed_graphs() {
    let envelope = support::envelope(SOURCE);
    let sequence = &envelope.project.sequences[0];
    assert_eq!(sequence.applies.len(), 1);
    let apply = &sequence.applies[0];
    assert!(matches!(apply.target, ApplyTarget::Layer { .. }));
    assert_eq!(apply.stages.len(), 2);
    assert!(matches!(
        apply.stages[0].operation,
        ApplyOperation::Color { .. }
    ));
    assert!(!apply.stages[1].enabled);
    assert_eq!(apply.mix.opacity, veac_ir::Animatable::constant(0.85));

    assert_eq!(envelope.project.relations.len(), 5);
    let transition = envelope
        .project
        .relations
        .iter()
        .find_map(|value| match &value.kind {
            RelationKind::Transition { transition, .. } => Some(transition),
            _ => None,
        })
        .unwrap();
    assert_eq!(transition.alignment, TransitionAlignment::Centered);
    assert_eq!(transition.kind, TransitionKind::Dissolve);
    assert_eq!(transition.duration.value, 600);
    assert!(envelope
        .project
        .relations
        .iter()
        .any(|value| matches!(value.kind, RelationKind::Matte { .. })));
    assert!(envelope
        .project
        .relations
        .iter()
        .any(|value| matches!(value.kind, RelationKind::Sidechain { .. })));
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn relation_and_apply_identity_is_repeatable() {
    assert_eq!(support::envelope(SOURCE), support::envelope(SOURCE));
}
