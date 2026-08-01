use super::*;

#[test]
fn item_set_apply_and_clip_mattes_share_one_cycle_graph() {
    let mut project = super::apply_matte_tests::project_with_apply("trk_scope");
    project.project.sequences[0].applies[0].target = ApplyTarget::ItemSet {
        item_ids: vec![ItemId::new("itm_scope").unwrap()],
    };
    project
        .project
        .relations
        .push(super::apply_matte_tests::matte(
            RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap()),
        ));
    project
        .project
        .relations
        .push(clip_matte("rel_mixed_cycle", "itm_scope", "itm_video"));
    assert_code(&validation_codes(&project), "MATTE_CYCLE");
}

#[test]
fn branched_matte_dependencies_do_not_report_a_cycle() {
    let mut project = super::composition_tests::project_with_matte();
    let mut branch = project.project.sequences[0].tracks[0].clone();
    branch.id = TrackId::new("trk_branch").unwrap();
    branch.order = 3;
    branch.clips[0].id = ItemId::new("itm_branch").unwrap();
    project.project.sequences[0].tracks.push(branch);
    project
        .project
        .relations
        .push(clip_matte("rel_branch", "itm_matte", "itm_branch"));
    let codes = validate(&project)
        .err()
        .map(|errors| errors.into_diagnostics())
        .unwrap_or_default();
    assert!(!codes
        .iter()
        .any(|diagnostic| diagnostic.code == "MATTE_CYCLE"));
}

fn clip_matte(id: &str, producer: &str, consumer: &str) -> Relation {
    Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new(producer).unwrap()),
            consumer: RelationEndpoint::item(ItemId::new(consumer).unwrap()),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Alpha,
                invert: false,
            },
        },
    }
}
