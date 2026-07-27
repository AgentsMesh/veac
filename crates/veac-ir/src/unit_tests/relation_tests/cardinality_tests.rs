use super::*;

#[test]
fn matte_consumer_has_one_canonical_projection() {
    let mut project = relation_project();
    let mut duplicate = relation_mut(&mut project, "rel_matte").clone();
    duplicate.id = RelationId::new("rel_matte_duplicate").unwrap();
    let RelationKind::Matte { parameters, .. } = &mut duplicate.kind else {
        unreachable!()
    };
    parameters.invert = true;
    project.project.relations.push(duplicate);

    assert_code(&project, "MULTIPLE_RELATION_PROJECTION");
}

#[test]
fn sidechain_target_has_one_canonical_projection() {
    let mut project = relation_project();
    let mut duplicate = relation_mut(&mut project, "rel_sidechain").clone();
    duplicate.id = RelationId::new("rel_sidechain_duplicate").unwrap();
    let RelationKind::Sidechain { parameters, .. } = &mut duplicate.kind else {
        unreachable!()
    };
    parameters.ratio = 8.0;
    project.project.relations.push(duplicate);

    assert_code(&project, "MULTIPLE_RELATION_PROJECTION");
}
