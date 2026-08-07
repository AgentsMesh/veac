use veac_ir::{AnnotationPayload, AnnotationSpan, AnnotationTarget, ClipSource, MulticamSyncBasis};

use super::support;

#[path = "multicam_annotation_v6/coverage.rs"]
mod coverage;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let wide = video_resource(
        identifier("wide"), resource_file("assets/wide.mp4"),
        sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        stream_intent(stream_auto(), stream_disabled())
    );
    let close = video_resource(
        identifier("close"), resource_file("assets/close.mp4"),
        sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        stream_intent(stream_auto(), stream_disabled())
    );
    let wide_angle = multicam_angle(identifier("wide"), wide, 0s);
    let close_angle = multicam_angle(identifier("close"), close, 0.25s);
    let group = multicam_group(
        identifier("interview"), multicam_sync(multicam_sync_audio(), wide_angle),
        [wide_angle, close_angle]
    );
    let clip = item(
        identifier("edit"), item_enabled(), during(0s, 4s),
        source_multicam(group, [
            multicam_switch(wide_angle, during(0s, 2s)),
            multicam_switch(close_angle, during(2s, 2s))
        ]),
        source_timing_native()
    );
    let track = video_layer(
        identifier("video"), 0, placement_free(),
        track_state(track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked()),
        track_routing_default()
    ).with_item(clip);
    let sequence = sequence(
        identifier("main"), "多机位主时间线",
        sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000)
    ).with_layer(track);
    let highlight = annotation(
        identifier("highlight"), annotation_target_item(clip),
        annotation_range(during(0.5s, 1.5s)),
        annotation_highlight(92%, "核心观点", ["后置证据", "前置证据"]),
        annotation_provenance_present(annotation_provenance(
            "veac.analysis.v1",
            sha256("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"),
            sha256("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")
        ))
    );
    let language = annotation(
        identifier("language"), annotation_target_resource(wide), annotation_untimed(),
        annotation_language([
            language_confidence("zh-Hans", 70%),
            language_confidence("en-US", 90%)
        ]),
        annotation_provenance_none()
    );
    let marker = annotation(
        identifier("group-marker"), annotation_target_multicam(group), annotation_untimed(),
        annotation_marker("双机位访谈", marker_color_present(#22aa66ff)),
        annotation_provenance_none()
    );
    project(identifier("demo"), project_settings(600))
        .with_resource(wide).with_resource(close)
        .with_multicam_group(group).with_sequence(sequence).entry(sequence)
        .with_annotation(highlight).with_annotation(language).with_annotation(marker)
}
"#;

#[test]
fn multicam_and_closed_annotations_lower_with_typed_references() {
    let envelope = support::envelope(SOURCE);
    let group = &envelope.project.multicam_groups[0];
    assert_eq!(group.sync.basis, MulticamSyncBasis::Audio);
    assert_eq!(group.angles.len(), 2);
    assert!(group
        .angles
        .iter()
        .any(|value| value.id == group.sync.reference_angle_id));
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    let ClipSource::Multicam { switches, .. } = &clip.source else {
        panic!("expected typed multicam source")
    };
    assert_eq!(switches.len(), 2);
    assert_eq!(switches[1].range.start.value, 1200);

    assert_eq!(envelope.project.annotations.len(), 3);
    let highlight = envelope
        .project
        .annotations
        .iter()
        .find(|value| matches!(value.payload, AnnotationPayload::Highlight { .. }))
        .unwrap();
    assert!(matches!(highlight.target, AnnotationTarget::Clip { .. }));
    assert!(matches!(highlight.span, AnnotationSpan::Range { .. }));
    let AnnotationPayload::Highlight { evidence, .. } = &highlight.payload else {
        unreachable!()
    };
    assert_eq!(evidence, &["后置证据", "前置证据"]);
    assert_eq!(
        highlight.provenance.as_ref().unwrap().producer,
        "veac.analysis.v1"
    );
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn language_scores_are_canonicalized_without_losing_typed_tags() {
    let envelope = support::envelope(SOURCE);
    let language = envelope
        .project
        .annotations
        .iter()
        .find_map(|value| match &value.payload {
            AnnotationPayload::Language { scores } => Some(scores),
            _ => None,
        })
        .unwrap();
    assert_eq!(language[0].language, "en-US");
    assert_eq!(language[1].language, "zh-Hans");
}
