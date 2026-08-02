use super::relation_support::relation_project;
use crate::{export_sequence, import_veac_extension, OtioItem, OtioTimeline};
use veac_ir::SequenceId;

#[test]
fn extension_roundtrip_preserves_ordered_relations_and_reports_loss() {
    let source = relation_project();
    let exported = export_sequence(&source, &SequenceId::new("seq_main").unwrap()).unwrap();
    let pointers: Vec<_> = exported
        .loss_report
        .losses
        .iter()
        .filter(|entry| entry.field == "kind" && entry.pointer.contains("/relations/"))
        .map(|entry| (entry.pointer.as_str(), entry.preserved_in_extension))
        .collect();
    assert_eq!(
        pointers,
        [
            ("/project/relations/rel_z_first", true),
            ("/project/relations/rel_a_second", true),
        ]
    );
    assert!(exported.timeline.tracks.children.iter().all(|track| track
        .children
        .iter()
        .all(|item| !matches!(item, OtioItem::Transition { .. }))));

    let imported = import_veac_extension(&exported.timeline).unwrap();
    assert_eq!(imported.sequence, source.project.sequences[0]);
    assert_eq!(imported.relations, source.project.relations);
}

#[test]
fn extension_v2_and_missing_relations_are_rejected() {
    let source = relation_project();
    let exported = export_sequence(&source, &SequenceId::new("seq_main").unwrap()).unwrap();
    let mut v2 = exported.timeline.clone();
    v2.metadata.get_mut("veac").unwrap()["version"] = 2.into();
    assert!(import_veac_extension(&v2).is_err());

    let mut missing = exported.timeline;
    missing
        .metadata
        .get_mut("veac")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("relations");
    assert!(import_veac_extension(&missing)
        .unwrap_err()
        .to_string()
        .contains("missing field `relations`"));
}

#[test]
fn extension_rejects_dangling_cross_scope_and_duplicate_relations() {
    let source = relation_project();
    let exported = export_sequence(&source, &SequenceId::new("seq_main").unwrap()).unwrap();

    let mut dangling = exported.timeline.clone();
    dangling.metadata.get_mut("veac").unwrap()["relations"][0]["kind"]["to"]["item_id"] =
        "itm_missing".into();
    assert_invalid_graph(&dangling, "RELATION_ENDPOINT");

    let mut cross_scope = exported.timeline.clone();
    cross_scope.metadata.get_mut("veac").unwrap()["relations"][0]["sequence_id"] =
        "seq_other".into();
    assert_invalid_graph(&cross_scope, "RELATION_SEQUENCE");

    let mut duplicate = exported.timeline;
    let copy = duplicate.metadata["veac"]["relations"][0].clone();
    duplicate.metadata.get_mut("veac").unwrap()["relations"]
        .as_array_mut()
        .unwrap()
        .push(copy);
    assert_invalid_graph(&duplicate, "DUPLICATE_RELATION_ID");
}

fn assert_invalid_graph(timeline: &OtioTimeline, code: &str) {
    let error = import_veac_extension(timeline).unwrap_err().to_string();
    assert!(error.contains("invalid VEAC relation graph"), "{error}");
    assert!(error.contains(code), "{error}");
}
