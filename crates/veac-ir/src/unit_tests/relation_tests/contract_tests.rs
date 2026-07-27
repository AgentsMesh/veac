use super::*;

#[test]
fn every_relation_kind_is_canonical_and_round_trips() {
    let project = relation_project();
    validate(&project).unwrap();
    let json = canonical_json(&project).unwrap();
    assert!(json.contains("rel_transition"));
    assert!(json.contains("rel_matte"));
    assert!(json.contains("rel_sidechain"));
    assert!(json.contains("rel_group"));
    assert!(json.contains("rel_primary"));
    assert_eq!(decode_canonical_json(&json).unwrap(), project);

    let schema = project_json_schema().unwrap();
    let required = schema["$defs"]["Project"]["required"].as_array().unwrap();
    assert!(required.iter().any(|field| field == "relations"));
    let text = serde_json::to_string(&schema).unwrap();
    assert!(text.contains("RelationEndpoint"));
    assert!(text.contains("SidechainRelationParameters"));
}

#[test]
fn sidechain_bus_endpoint_round_trips_without_clip_projection() {
    let mut project = relation_project();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[1].routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialog").unwrap(),
    };
    let parameters = sidechain_parameters();
    relation_mut(&mut project, "rel_sidechain").kind = RelationKind::Sidechain {
        key: RelationEndpoint::bus(BusId::new("bus_dialog").unwrap()),
        target: item("itm_video"),
        parameters,
    };
    validate(&project).unwrap();
    let json = canonical_json(&project).unwrap();
    assert!(json.contains(r#""bus_id":"bus_dialog""#));
    let value = serde_json::to_value(&project).unwrap();
    assert!(
        value["project"]["sequences"][0]["tracks"][0]["clips"][0]["audio"]
            .get("sidechain")
            .is_none()
    );
}

#[test]
fn schema_version_marks_the_relation_table_as_a_clean_break() {
    let project = relation_project();
    assert_eq!(project.schema_version, 3);
    assert_eq!(project.min_reader_version, 3);
    let mut json = serde_json::to_value(project).unwrap();
    json["project"].as_object_mut().unwrap().remove("relations");
    assert!(decode_canonical_json(&json.to_string()).is_err());
}

#[test]
fn legacy_sequence_membership_fields_are_rejected() {
    for field in ["groups", "av_links"] {
        let mut value = serde_json::to_value(linked_project()).unwrap();
        value["project"]["sequences"][0]
            .as_object_mut()
            .unwrap()
            .insert(field.to_owned(), serde_json::json!([]));
        let error = decode_canonical_json(&value.to_string()).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
        assert!(error.to_string().contains(field));
    }
}

#[test]
fn relation_only_av_link_fixture_validates_and_round_trips() {
    let project = linked_project();
    validate(&project).unwrap();
    assert!(matches!(
        &project.project.relations[0].kind,
        RelationKind::AvLink { .. }
    ));
    let json = canonical_json(&project).unwrap();
    assert_eq!(decode_canonical_json(&json).unwrap(), project);
}
