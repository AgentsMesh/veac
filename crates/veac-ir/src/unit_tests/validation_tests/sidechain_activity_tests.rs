use super::*;

#[test]
fn sidechain_sources_report_type_self_and_inactive_failures() {
    let valid = sidechain_project("trk_audio", "itm_video");
    assert!(validate(&valid).is_ok());

    let mut wrong_type = valid.clone();
    wrong_type.project.sequences[0].tracks[1].kind = TrackKind::Visual;
    assert_code(&validation_codes(&wrong_type), "SIDECHAIN_SOURCE_TYPE");

    let self_dependent = sidechain_project("trk_video", "itm_video");
    assert_code(
        &validation_codes(&self_dependent),
        "SIDECHAIN_SELF_DEPENDENCY",
    );

    let mut inactive = valid;
    inactive.project.sequences[0].tracks[1].state.muted = true;
    assert_code(&validation_codes(&inactive), "SIDECHAIN_SOURCE_INACTIVE");
}

#[test]
fn inactive_targets_are_dormant_and_audio_tracks_have_implicit_properties() {
    let mut project = sidechain_project("trk_video", "itm_audio");
    assert!(validate(&project).is_ok());
    project.project.sequences[0].tracks[1].clips[0].enabled = false;
    let codes = codes(&project);
    assert!(!codes
        .iter()
        .any(|code| code.starts_with("SIDECHAIN_SOURCE_")));

    let mut wrong_target = sidechain_project("trk_audio", "itm_caption");
    wrong_target.project.sequences[0].tracks[2].clips[0].audio = None;
    assert_code(&validation_codes(&wrong_target), "SIDECHAIN_TARGET_TYPE");
}

#[test]
fn reciprocal_sidechains_are_valid_pre_sidechain_controls() {
    let mut project = sidechain_project("trk_audio", "itm_video");
    project
        .project
        .relations
        .push(sidechain("rel_sidechain_reverse", "trk_video", "itm_audio"));
    assert!(validate(&project).is_ok());
}

fn sidechain_project(key: &str, target: &str) -> ProjectEnvelope {
    let mut project = crate::test_support::linked_project();
    project
        .project
        .relations
        .push(sidechain("rel_sidechain_activity", key, target));
    project
}

fn sidechain(id: &str, key: &str, target: &str) -> Relation {
    Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(TrackId::new(key).unwrap()),
            target: RelationEndpoint::item(ItemId::new(target).unwrap()),
            parameters: SidechainRelationParameters {
                threshold_db: -18.0,
                ratio: 4.0,
                attack_ms: 20.0,
                release_ms: 200.0,
                active_range: None,
            },
        },
    }
}

fn codes(project: &ProjectEnvelope) -> Vec<String> {
    validate(project)
        .err()
        .map(|errors| {
            errors
                .into_diagnostics()
                .into_iter()
                .map(|diagnostic| diagnostic.code)
                .collect()
        })
        .unwrap_or_default()
}
