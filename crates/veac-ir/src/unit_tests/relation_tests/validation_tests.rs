use super::*;

#[test]
fn relation_identity_and_sequence_ownership_are_project_scoped() {
    let mut duplicate = relation_project();
    duplicate
        .project
        .relations
        .push(duplicate.project.relations[0].clone());
    assert_code(&duplicate, "DUPLICATE_RELATION_ID");

    let mut double_claim = relation_project();
    let mut second = relation_mut(&mut double_claim, "rel_transition").clone();
    second.id = RelationId::new("rel_second_identity").unwrap();
    double_claim.project.relations.push(second);
    assert_code(&double_claim, "MULTIPLE_RELATION_PROJECTION");

    let mut missing = relation_project();
    missing.project.relations[0].sequence_id = SequenceId::new("seq_missing").unwrap();
    assert_code(&missing, "RELATION_SEQUENCE_NOT_FOUND");
}

#[test]
fn endpoint_types_existence_and_transition_context_are_closed() {
    let mut wrong_type = relation_project();
    if let RelationKind::Transition { from, .. } =
        &mut relation_mut(&mut wrong_type, "rel_transition").kind
    {
        *from = RelationEndpoint::track(track_id("trk_video"));
    }
    assert_code(&wrong_type, "RELATION_ENDPOINT_TYPE");

    let mut missing = relation_project();
    if let RelationKind::Group { members } = &mut relation_mut(&mut missing, "rel_group").kind {
        members[0] = item("itm_missing");
    }
    assert_code(&missing, "RELATION_ENDPOINT_NOT_FOUND");

    let mut context = relation_project();
    if let RelationKind::Transition { to, .. } =
        &mut relation_mut(&mut context, "rel_transition").kind
    {
        *to = item("itm_caption");
    }
    assert_code(&context, "RELATION_CONTEXT");
}

#[test]
fn membership_relations_enforce_single_ownership() {
    let mut groups = relation_project();
    let mut duplicate = relation_mut(&mut groups, "rel_group").clone();
    duplicate.id = RelationId::new("rel_group_duplicate").unwrap();
    groups.project.relations.push(duplicate);
    assert_code(&groups, "MULTIPLE_GROUP_MEMBERSHIP");

    let mut links = relation_project();
    let mut duplicate = relation_mut(&mut links, "rel_primary").clone();
    duplicate.id = RelationId::new("rel_primary_duplicate").unwrap();
    links.project.relations.push(duplicate);
    assert_code(&links, "MULTIPLE_AV_LINK_MEMBERSHIP");
}

#[test]
fn sidechain_active_range_is_target_item_local() {
    let mut project = relation_project();
    let invalid = Some(range(500, 200));
    if let RelationKind::Sidechain { parameters, .. } =
        &mut relation_mut(&mut project, "rel_sidechain").kind
    {
        parameters.active_range = invalid;
    }
    assert_code(&project, "SIDECHAIN_RANGE");
}

#[test]
fn relation_members_are_nonempty_and_unique() {
    let mut duplicate = relation_project();
    if let RelationKind::Group { members } = &mut relation_mut(&mut duplicate, "rel_group").kind {
        members[1] = members[0].clone();
    }
    assert_code(&duplicate, "DUPLICATE_RELATION_MEMBER");

    let mut empty = relation_project();
    if let RelationKind::AvLink { audio, .. } = &mut relation_mut(&mut empty, "rel_primary").kind {
        audio.clear();
    }
    assert_code(&empty, "RELATION_MEMBERS");
}
