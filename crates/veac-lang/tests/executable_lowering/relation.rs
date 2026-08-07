use veac_ir::{RelationKind, TrackKind, TransitionAlignment, TransitionKind};
use veac_lang::program::DomainOperationId as Op;

use super::support;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let outgoing = item(identifier("out"), item_enabled(), during(0s, 1000001us),
        source_generated(generator_solid(#ff0000ff)), source_timing_native());
    let incoming = item(identifier("in"), item_enabled(), during(1s, 1s),
        source_generated(generator_solid(#0000ffff)), source_timing_native());
    let cross = relation_transition(
        identifier("cross"), outgoing, incoming, transition_dissolve(1us));
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
        track_routing_default()).with_item(incoming).with_item(outgoing);
    let audio = audio_layer(identifier("audio"), 1, placement_free(), state,
        track_routing_default());
    let timeline = sequence(identifier("main"), "微秒转场",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(visual).with_relation(cross).with_layer(audio);
    project(identifier("demo"), project_settings(3000000))
        .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn transition_lowers_to_stable_centered_dissolve_and_endpoint_ids() {
    let envelope = support::envelope(SOURCE);
    let project = &envelope.project;
    assert_eq!(project.timebase, 3_000_000);
    assert_eq!(project.relations.len(), 1);
    let relation = &project.relations[0];
    assert!(relation.id.as_str().starts_with("rel_"));
    assert_eq!(relation.sequence_id, project.sequences[0].id);
    let RelationKind::Transition {
        from,
        to,
        transition,
    } = &relation.kind
    else {
        panic!("expected transition relation")
    };
    assert_eq!(
        from.item_id(),
        Some(&project.sequences[0].tracks[0].clips[0].id)
    );
    assert_eq!(
        to.item_id(),
        Some(&project.sequences[0].tracks[0].clips[1].id)
    );
    assert!(matches!(transition.kind, TransitionKind::Dissolve));
    assert_eq!(transition.alignment, TransitionAlignment::Centered);
    assert_eq!(transition.duration.value, 3);
    assert_eq!(transition.duration.timescale, 3_000_000);
}

#[test]
fn relation_attachment_does_not_change_track_or_clip_order() {
    let envelope = support::envelope(SOURCE);
    let sequence = &envelope.project.sequences[0];
    assert_eq!(sequence.tracks.len(), 2);
    assert_eq!(sequence.tracks[0].kind, TrackKind::Visual);
    assert_eq!(sequence.tracks[0].order, 0);
    assert_eq!(sequence.tracks[1].kind, TrackKind::Audio);
    assert_eq!(sequence.tracks[1].order, 1);
    assert_eq!(sequence.tracks[0].clips[0].record_range.start.value, 0);
    assert_eq!(
        sequence.tracks[0].clips[1].record_range.start.value,
        3_000_000
    );
}

#[test]
fn relation_provenance_is_indexed_by_stable_relation_id() {
    let envelope = support::envelope(SOURCE);
    let sequence = &envelope.project.sequences[0];
    let relation = &envelope.project.relations[0];
    let veac_ir::SequenceAuthorship::Veac {
        entity, relations, ..
    } = sequence.authorship.as_ref().unwrap()
    else {
        panic!("expected VEAC authorship")
    };
    let provenance = relations;
    assert_eq!(provenance.len(), 1);
    let value = &provenance
        .iter()
        .find(|value| value.relation_id == relation.id)
        .unwrap()
        .entity;
    assert_eq!(
        value
            .logical_path
            .iter()
            .map(|part| part.as_str())
            .collect::<Vec<_>>(),
        vec!["demo", "sequence", "main", "relation", "cross"]
    );
    assert_eq!(
        value.events[0].kind,
        veac_ir::AuthorshipEventKind::Constructor
    );
    assert_eq!(value.events[0].operation.0, Op::RelationTransition.opcode());
    assert!(entity.events.iter().any(|event| {
        event.kind == veac_ir::AuthorshipEventKind::Update
            && event.operation.0 == Op::SequenceWithRelation.opcode()
    }));
}

#[test]
fn transition_project_is_canonical_roundtrip_deterministic() {
    let first = support::envelope(SOURCE);
    let second = support::envelope(SOURCE);
    assert_eq!(first, second);
    veac_ir::validate(&first).unwrap();
    let json = veac_ir::canonical_json(&first).unwrap();
    assert_eq!(veac_ir::decode_canonical_json(&json).unwrap(), first);
}

#[test]
fn non_positive_and_mismatched_dissolves_fail_closed() {
    let zero = SOURCE.replace("transition_dissolve(1us)", "transition_dissolve(0us)");
    let error = support::error(&zero);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR");
    assert!(error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"));

    let mismatch = SOURCE.replace("transition_dissolve(1us)", "transition_dissolve(2us)");
    let error = support::error(&mismatch);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR");
    assert!(error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"));
}
