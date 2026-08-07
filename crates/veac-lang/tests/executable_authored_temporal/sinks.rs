use veac_ir::Animatable;
use veac_lang::program::{build_source, prepare_source};
use veac_lang::source_edit::{BodySite, SourceNodeRef, SourceTemporalProperty as SourceProperty};

use super::support::SINK_MATRIX;

const DECLARATIONS: &str = r#"
animate visual-position on clip(@matrix, @main, @visual, @visual-clip) {
  point(progress * 10px, progress * 20px)
}
animate visual-scale on clip(@matrix, @main, @visual, @visual-clip) {
  vector(1.0 + progress, 1.0 + progress)
}
animate visual-rotation on clip(@matrix, @main, @visual, @visual-clip) {
  progress * 90deg
}
animate visual-crop on clip(@matrix, @main, @visual, @visual-clip) {
  rect(progress, 0.0, 1.0, 1.0)
}
animate visual-opacity on clip(@matrix, @main, @visual, @visual-clip) {
  clamp(progress, 0.0, 1.0)
}
animate audio-gain on clip(@matrix, @main, @audio, @audio-clip) {
  1.0 - progress * 0.5
}
animate audio-pan on clip(@matrix, @main, @audio, @audio-clip) {
  progress * 2.0 - 1.0
}
"#;

#[test]
fn all_closed_authored_sinks_attach_typed_canonical_bindings() {
    let source = format!("{DECLARATIONS}\n{SINK_MATRIX}");
    let built = build_source(&source).unwrap();
    let envelope = built.envelope();
    let sequence = &envelope.project.sequences[0];
    let visual = sequence.tracks[0].clips[0].visual.as_ref().unwrap();
    for value in [
        visual.transform.position.binding_id(),
        visual.transform.scale.binding_id(),
        visual.transform.rotation_degrees.binding_id(),
        visual.transform.crop.as_ref().unwrap().binding_id(),
        visual.opacity.binding_id(),
    ] {
        assert!(value.is_some());
    }
    let audio = sequence.tracks[1].clips[0].audio.as_ref().unwrap();
    assert!(matches!(audio.gain, Animatable::Binding { .. }));
    assert!(matches!(audio.pan, Animatable::Binding { .. }));
    assert_eq!(envelope.temporal.bindings.len(), 7);
    assert!(veac_ir::validate(envelope).is_ok());
}

#[test]
fn source_index_publishes_every_closed_temporal_body_site() {
    let source = format!("{DECLARATIONS}\n{SINK_MATRIX}");
    let index = prepare_source(&source).unwrap().source_index().unwrap();
    for property in [
        SourceProperty::VisualPosition,
        SourceProperty::VisualScale,
        SourceProperty::VisualRotation,
        SourceProperty::VisualCrop,
        SourceProperty::VisualOpacity,
    ] {
        let target = SourceNodeRef::temporal(
            "main.veac",
            "matrix",
            "main",
            "visual",
            "visual-clip",
            property,
        );
        assert!(index
            .body(&target, BodySite::TemporalAnimation { property })
            .is_some());
    }
    for property in [SourceProperty::AudioGain, SourceProperty::AudioPan] {
        let target = SourceNodeRef::temporal(
            "main.veac",
            "matrix",
            "main",
            "audio",
            "audio-clip",
            property,
        );
        assert!(index
            .body(&target, BodySite::TemporalAnimation { property })
            .is_some());
    }
}

#[test]
fn authored_sink_type_mismatch_is_rejected_before_publish() {
    let source = format!(
        "animate visual-position on clip(@matrix, @main, @visual, @visual-clip) \
         {{ progress }}\n{SINK_MATRIX}"
    );
    let error = build_source(&source).unwrap_err();
    assert!(error.as_slice()[0]
        .message
        .contains("EXECUTABLE_TEMPORAL_SINK_TYPE"));
}

#[test]
fn authored_crop_cannot_create_an_absent_optional_leaf() {
    let source = SINK_MATRIX.replace(
        "crop_animated(rect_constant(rect(0.0, 0.0, 1.0, 1.0)))",
        "crop_none()",
    );
    let source = format!(
        "animate visual-crop on clip(@matrix, @main, @visual, @visual-clip) \
         {{ rect(progress, 0.0, 1.0, 1.0) }}\n{source}"
    );
    let error = build_source(&source).unwrap_err();
    assert!(error.as_slice()[0]
        .message
        .contains("EXECUTABLE_TEMPORAL_OPTIONAL_SINK"));
}
