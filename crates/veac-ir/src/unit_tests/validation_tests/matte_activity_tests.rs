use super::*;

#[test]
fn matte_structure_and_live_source_have_distinct_diagnostics() {
    let mut wrong_consumer = super::composition_tests::project_with_matte();
    wrong_consumer.project.sequences[0].tracks[0].kind = TrackKind::Audio;
    assert_code(&validation_codes(&wrong_consumer), "MATTE_CONSUMER_TYPE");

    let mut wrong_source = super::composition_tests::project_with_matte();
    wrong_source.project.sequences[0].tracks[2].kind = TrackKind::Audio;
    assert_code(&validation_codes(&wrong_source), "MATTE_SOURCE_TYPE");

    let mut inactive = super::composition_tests::project_with_matte();
    inactive.project.sequences[0].tracks[2].state.enabled = false;
    assert_code(&validation_codes(&inactive), "MATTE_SOURCE_INACTIVE");
}

#[test]
fn inactive_consumers_do_not_require_an_active_matte_source() {
    let mut project = super::composition_tests::project_with_matte();
    project.project.sequences[0].tracks[0].state.enabled = false;
    project.project.sequences[0].tracks[2].state.enabled = false;
    let codes = codes(&project);
    assert!(!codes.iter().any(|code| code == "MATTE_SOURCE_INACTIVE"));

    let mut solo = super::composition_tests::project_with_matte();
    solo.project.sequences[0].tracks[0].state.solo = true;
    assert_code(&validation_codes(&solo), "MATTE_SOURCE_INACTIVE");
}

#[test]
fn disabled_apply_stages_do_not_activate_apply_mattes() {
    let mut project = super::apply_matte_tests::project_with_apply("trk_scope");
    project.project.sequences[0].tracks[0].state.enabled = false;
    project
        .project
        .relations
        .push(super::apply_matte_tests::matte(
            RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap()),
        ));
    project.project.sequences[0].applies[0].enabled = false;
    assert_no_inactive_source(&project);

    project.project.sequences[0].applies[0].enabled = true;
    project.project.sequences[0].applies[0].stages[0].enabled = false;
    assert_no_inactive_source(&project);

    let mut effect = project.project.sequences[0].tracks[0].clips[0].effects[0].clone();
    effect.enabled = false;
    project.project.sequences[0].applies[0].stages[0].enabled = true;
    project.project.sequences[0].applies[0].stages[0].operation = ApplyOperation::Effect { effect };
    assert_no_inactive_source(&project);

    let ApplyOperation::Effect { effect } =
        &mut project.project.sequences[0].applies[0].stages[0].operation
    else {
        unreachable!()
    };
    effect.enabled = true;
    assert_code(&validation_codes(&project), "MATTE_SOURCE_INACTIVE");
}

fn assert_no_inactive_source(project: &ProjectEnvelope) {
    assert!(!codes(project)
        .iter()
        .any(|code| code == "MATTE_SOURCE_INACTIVE"));
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
