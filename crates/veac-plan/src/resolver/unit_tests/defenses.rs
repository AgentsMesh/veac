use super::super::{
    error::{canonical_errors, internal_hash_error, normalize_diagnostics},
    material::InputUsage,
    PlanResolver,
};
use super::support::*;
use crate::{canonical::*, *};

pub(super) fn direct(value: &ProjectEnvelope) -> PlanResolver<'_> {
    PlanResolver::new(
        value,
        &value.project.render_configs[0],
        "semantic",
        "snapshot",
    )
}

#[test]
fn diagnostic_helpers_sort_deduplicate_and_map_internal_errors() {
    let mut invalid = project();
    invalid.schema = "invalid".to_owned();
    let validation = validate(&invalid).unwrap_err();
    let mapped = canonical_errors(validation);
    assert_eq!(
        mapped.diagnostics()[0].kind,
        ResolutionErrorKind::CanonicalValidation
    );
    let canonical = canonical_json(&invalid).unwrap_err();
    assert_eq!(
        internal_hash_error(canonical).diagnostics()[0].code,
        "CANONICAL_HASH_FAILED"
    );

    let first = ResolutionDiagnostic::new(
        ResolutionErrorKind::InternalInvariant,
        "B",
        None,
        "/z",
        "second",
    );
    let second = ResolutionDiagnostic::new(
        ResolutionErrorKind::InternalInvariant,
        "A",
        None,
        "/a",
        "first",
    );
    let mut diagnostics = vec![first, second.clone(), second];
    normalize_diagnostics(&mut diagnostics);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].code, "A");

    let value = project();
    let mut resolver = direct(&value);
    resolver.push_internal("INTERNAL_TEST", "itm_test".into(), "failure".into());
    assert_eq!(resolver.diagnostics[0].code, "INTERNAL_TEST");
}

#[test]
fn direct_material_defenses_cover_invalid_identity_inventory_and_kind() {
    let mut mismatch = project();
    mismatch.project.materials[0].identity = Some(identity('c'));
    let mut resolver = direct(&mismatch);
    assert!(resolver
        .material_input(
            &MaterialId::new("med_video").unwrap(),
            InputUsage {
                video: true,
                ..InputUsage::default()
            },
        )
        .is_none());
    assert_eq!(
        resolver.diagnostics[0].code,
        "MATERIAL_PROBE_IDENTITY_MISMATCH"
    );

    let mut inventory = project();
    inventory.project.materials[0]
        .probe
        .as_mut()
        .unwrap()
        .selected_video_stream = Some(StreamSelection {
        global_index: 99,
        type_index: 0,
    });
    let mut resolver = direct(&inventory);
    resolver.material_input(
        &MaterialId::new("med_video").unwrap(),
        InputUsage::default(),
    );
    assert_eq!(resolver.diagnostics[0].code, "STREAM_SELECTION_MISMATCH");

    let mut no_video = project();
    no_video.project.materials[0]
        .probe
        .as_mut()
        .unwrap()
        .selected_video_stream = None;
    let mut resolver = direct(&no_video);
    let id = MaterialId::new("med_video").unwrap();
    resolver.material_input(
        &id,
        InputUsage {
            video: true,
            ..InputUsage::default()
        },
    );
    resolver.material_input(
        &id,
        InputUsage {
            font: true,
            ..InputUsage::default()
        },
    );
    let codes: Vec<_> = resolver
        .diagnostics
        .iter()
        .map(|item| item.code.as_str())
        .collect();
    assert!(codes.contains(&"REQUIRED_VIDEO_STREAM_MISSING"));
    assert!(codes.contains(&"FONT_INPUT_KIND"));
}

#[test]
fn direct_id_sequence_mapping_and_transition_defenses_are_total() {
    let value = project();
    let mut resolver = direct(&value);
    let invalid_id: MaterialId = serde_json::from_str("\"med_bad/slash\"").unwrap();
    assert!(resolver
        .material_input(&invalid_id, InputUsage::default())
        .is_none());
    assert_eq!(resolver.diagnostics[0].code, "PLAN_INPUT_ID");

    let missing = SequenceId::new("seq_missing").unwrap();
    resolver.resolve_sequence(&missing);
    assert!(resolver
        .diagnostics
        .iter()
        .any(|item| item.code == "SEQUENCE_NOT_FOUND"));
    let main = SequenceId::new("seq_main").unwrap();
    resolver.visiting.insert(main.clone());
    resolver.resolve_sequence(&main);
    assert!(resolver
        .diagnostics
        .iter()
        .any(|item| item.code == "SEQUENCE_CYCLE"));
    resolver.visiting.clear();
    resolver.resolve_sequence(&main);
    resolver.resolve_sequence(&main);
    assert!(resolver
        .source_order(usize::MAX, "itm_test".into())
        .is_none());

    let mut invalid_clip = media_clip("itm_invalid", "med_video", 0);
    invalid_clip.source_mapping = None;
    assert!(resolver
        .resolve_clip(
            &main,
            &invalid_clip,
            TrackKind::Video,
            EffectiveTrackState {
                include_in_render: true,
                visual_enabled: true,
                audio_enabled: false,
            },
            0,
        )
        .is_none());

    let outgoing = media_clip("itm_outgoing", "med_video", 0);
    let incoming = media_clip("itm_incoming", "med_video", 600);
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: RationalTime::new(1, 1).unwrap(),
        alignment: TransitionAlignment::Centered,
    };
    assert!(resolver
        .build_transition(
            &veac_ir::RelationId::new("rel_test").unwrap(),
            &outgoing,
            &incoming,
            &transition,
        )
        .is_none());
}
