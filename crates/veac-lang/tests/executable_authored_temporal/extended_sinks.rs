use veac_ir::{
    Animatable, ApplyOperation, Effect, EffectParameter, TemporalClock, TemporalClockOwner,
};
use veac_lang::program::{build_source, prepare_source};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourceTemporalProperty,
};

const SOURCE: &str = include_str!("../fixtures/authored_sink_matrix.veac");

#[test]
fn every_nested_canonical_sink_attaches_from_typed_source() {
    let built = build_source(SOURCE).unwrap();
    let envelope = built.envelope();
    assert_eq!(envelope.temporal.bindings.len(), 19);
    let sequence = &envelope.project.sequences[0];
    let picture = &sequence.tracks[0].clips[0];
    let mask = &picture.visual.as_ref().unwrap().masks[0];
    assert_mask(mask);
    assert!(curve(&picture.effects[0].effect, EffectParameter::Radius));
    let title = &sequence.tracks[1].clips[0];
    let animation = match &title.source {
        veac_ir::ClipSource::Text { style, .. } => style.animation.as_ref().unwrap(),
        _ => panic!("expected text clip"),
    };
    for bound in [
        animation.transform.position_offset.binding_id(),
        animation.transform.scale.binding_id(),
        animation.transform.rotation_degrees.binding_id(),
        animation.reveal.binding_id(),
        animation.highlight.as_ref().unwrap().progress.binding_id(),
        animation.opacity.binding_id(),
    ] {
        assert!(bound.is_some());
    }
    let apply = &sequence.applies[0];
    assert!(apply.mix.opacity.binding_id().is_some());
    assert_mask(&apply.mix.masks[0]);
    let ApplyOperation::Effect { effect } = &apply.stages[0].operation else {
        panic!("expected apply effect")
    };
    assert!(curve(&effect.effect, EffectParameter::Radius));
    assert!(veac_ir::validate(envelope).is_ok());
}

#[test]
fn apply_sinks_use_only_the_static_sequence_owner() {
    let built = build_source(SOURCE).unwrap();
    let envelope = built.envelope();
    let sequence_id = &envelope.project.sequences[0].id;
    let apply_bindings = &envelope.temporal.bindings[12..];
    assert_eq!(apply_bindings.len(), 7);
    for binding in apply_bindings {
        assert_eq!(binding.clocks.len(), 1);
        assert_eq!(binding.clocks[0].clock, TemporalClock::SequenceTime);
        assert_eq!(
            binding.clocks[0].owner,
            TemporalClockOwner::Sequence {
                sequence_id: sequence_id.clone()
            }
        );
    }
}

#[test]
fn typed_target_mismatch_duplicate_and_missing_leaf_fail_closed() {
    let mismatch = SOURCE.replace("mask-position on clip-mask", "text-position on clip-mask");
    assert_eq!(
        prepare_source(&mismatch).unwrap_err().as_slice()[0].code,
        "PROGRAM_TEMPORAL_TARGET"
    );
    let declaration = "animate mask-position on clip-mask(\
      @sink-matrix, @main, @visual, @picture, 0) { vector(progress, progress) }";
    let duplicate = SOURCE.replacen("fn window", &format!("{declaration}\nfn window"), 1);
    assert_eq!(
        prepare_source(&duplicate).unwrap_err().as_slice()[0].code,
        "PROGRAM_TEMPORAL_SINK_DUPLICATE"
    );
    let missing = SOURCE.replace("@picture, 0)", "@picture, 9)");
    assert!(build_source(&missing).unwrap_err().as_slice()[0]
        .message
        .contains("EXECUTABLE_TEMPORAL_OPTIONAL_SINK"));
}

#[test]
fn nested_sink_identity_and_source_index_are_stable_and_specific() {
    let first = build_source(SOURCE).unwrap();
    let changed = SOURCE.replace("progress * 12.0", "progress * 10.0");
    let second = build_source(&changed).unwrap();
    assert_eq!(
        first.envelope().temporal.bindings[3].id,
        second.envelope().temporal.bindings[3].id
    );
    let index = prepare_source(SOURCE).unwrap().source_index().unwrap();
    let target = SourceNodeRef::temporal_clip_mask(
        "main.veac",
        ["sink-matrix", "main", "visual", "picture"],
        0,
        SourceTemporalProperty::MaskFeather,
    );
    assert!(index
        .body(
            &target,
            BodySite::TemporalAnimation {
                property: SourceTemporalProperty::MaskFeather
            }
        )
        .unwrap()
        .source
        .contains("12.0"));
}

#[test]
fn nested_sink_body_edit_rebuilds_from_veac_source_truth() {
    let directory = tempfile::tempdir().unwrap();
    let entry = directory.path().join("main.veac");
    std::fs::write(&entry, SOURCE).unwrap();
    let prepared = veac_lang::program::prepare_path(&entry).unwrap();
    let target = SourceNodeRef::temporal_clip_mask(
        "main.veac",
        ["sink-matrix", "main", "visual", "picture"],
        0,
        SourceTemporalProperty::MaskPosition,
    );
    let site = BodySite::TemporalAnimation {
        property: SourceTemporalProperty::MaskPosition,
    };
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_nested_temporal_body").unwrap(),
        prepared.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site,
        body: BodySource {
            source: "{ vector(progress, progress) }".to_owned(),
        },
    });
    let preview = veac_lang::program::apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert!(preview
        .source()
        .unwrap()
        .contains("vector(progress, progress)"));
    let mask = &preview.built.envelope().project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .masks[0];
    assert!(mask.position.binding_id().is_some());
    assert_eq!(std::fs::read_to_string(entry).unwrap(), SOURCE);
}

fn assert_mask(mask: &veac_ir::Mask) {
    for bound in [
        mask.position.binding_id(),
        mask.scale.binding_id(),
        mask.rotation_degrees.binding_id(),
        mask.feather_pixels.binding_id(),
        mask.expansion_pixels.binding_id(),
    ] {
        assert!(bound.is_some());
    }
}

fn curve(effect: &Effect, parameter: EffectParameter) -> bool {
    matches!(effect.curve(parameter), Some(Animatable::Binding { .. }))
}
