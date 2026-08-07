use super::*;

#[test]
fn typed_authorship_roundtrips_and_affects_semantic_identity() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].authorship =
        Some(entity(&["project", "main", "video", "first"]));
    let canonical = canonical_json(&project).unwrap();
    assert!(!canonical.contains("veac.provenance.v1"));
    assert!(!canonical.contains("\"metadata\""));
    assert_eq!(decode_canonical_json(&canonical).unwrap(), project);

    let first = semantic_hash(&project).unwrap();
    project.project.sequences[0].tracks[0].clips[0]
        .authorship
        .as_mut()
        .unwrap()
        .events[0]
        .operation = DomainOpcode(0x1002);
    assert_ne!(semantic_hash(&project).unwrap(), first);
}

#[test]
fn authorship_rejects_unknown_opcode_duplicate_path_and_invalid_span() {
    let mut project = sample_project();
    let authored = entity(&["project", "main", "same"]);
    project.project.materials[0].authorship = Some(authored.clone());
    project.project.sequences[0].tracks[0].clips[0].authorship = Some(authored);
    let event = &mut project.project.materials[0]
        .authorship
        .as_mut()
        .unwrap()
        .events[0];
    event.operation = DomainOpcode(0xffff);
    event.origin.span.start = 9;
    event.origin.span.end = 2;
    let codes = validation_codes(&project);
    for code in ["AUTHORSHIP_OPCODE", "AUTHORSHIP_PATH", "AUTHORSHIP_SITE"] {
        assert_code(&codes, code);
    }
}

#[test]
fn nested_unknown_authorship_fields_are_rejected_by_the_typed_abi() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].authorship =
        Some(entity(&["project", "main", "video", "first"]));
    let mut value = serde_json::to_value(project).unwrap();
    value["project"]["sequences"][0]["tracks"][0]["clips"][0]["authorship"]
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), true.into());
    assert!(matches!(
        decode_canonical_json(&value.to_string()),
        Err(CanonicalError::Json(_))
    ));
}

#[test]
fn sequence_authorship_must_exactly_name_sorted_owned_entities() {
    let mut project = sample_project();
    project.project.sequences[0].authorship = Some(SequenceAuthorship::Veac {
        entity: entity(&["project", "main"]),
        tracks: Vec::new(),
        relations: Vec::new(),
        applies: Vec::new(),
    });
    assert_code(&validation_codes(&project), "AUTHORSHIP_OWNERSHIP");
}

#[test]
fn project_authorship_exactly_covers_all_owned_entity_kinds() {
    let mut project = crate::test_support::multicam_project();
    project.project.annotations.push(Annotation {
        id: AnnotationId::new("ann_authored").unwrap(),
        target: AnnotationTarget::Project,
        span: AnnotationSpan::Untimed,
        payload: AnnotationPayload::Marker {
            label: "authored".to_owned(),
            color: None,
        },
        provenance: None,
    });
    project.project.authorship = Some(ProjectAuthorship {
        entity: entity(&["project"]),
        multicam_groups: vec![MulticamAuthorship {
            group_id: project.project.multicam_groups[0].id.clone(),
            entity: entity(&["project", "multicam"]),
        }],
        annotations: vec![AnnotationAuthorship {
            annotation_id: project.project.annotations[0].id.clone(),
            entity: entity(&["project", "annotation"]),
        }],
        deliveries: vec![DeliveryAuthorship {
            render_config_id: project.project.render_configs[0].id.clone(),
            entity: entity(&["project", "delivery"]),
        }],
    });
    validate(&project).unwrap();
}

#[test]
fn veac_sequence_authorship_exactly_covers_tracks_relations_and_applies() {
    let mut project = crate::test_support::linked_project();
    let relation_id = project.project.relations[0].id.clone();
    let sequence = &mut project.project.sequences[0];
    sequence.applies.push(super::apply_target_tests::base_apply(
        "apl_authored",
        "aps_authored",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    ));
    let mut tracks = sequence
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| TrackAuthorship {
            track_id: track.id.clone(),
            entity: entity(&["sequence", "track", &format!("track_{index}")]),
        })
        .collect::<Vec<_>>();
    tracks.sort_by(|left, right| left.track_id.cmp(&right.track_id));
    sequence.authorship = Some(SequenceAuthorship::Veac {
        entity: entity(&["sequence"]),
        tracks,
        relations: vec![RelationAuthorship {
            relation_id,
            entity: entity(&["sequence", "relation"]),
        }],
        applies: vec![ApplyAuthorship {
            apply_id: ApplyId::new("apl_authored").unwrap(),
            entity: entity(&["sequence", "apply"]),
        }],
    });
    validate(&project).unwrap();
}

#[test]
fn otio_authorship_requires_a_lowercase_sha256_digest() {
    let mut valid = sample_project();
    valid.project.sequences[0].authorship = Some(SequenceAuthorship::Otio {
        document_sha256: Sha256Digest::new("a".repeat(64)),
    });
    validate(&valid).unwrap();

    let mut invalid = valid;
    invalid.project.sequences[0].authorship = Some(SequenceAuthorship::Otio {
        document_sha256: Sha256Digest::new("not-a-sha256"),
    });
    assert_code(&validation_codes(&invalid), "AUTHORSHIP_OTIO_DIGEST");
}

fn entity(path: &[&str]) -> EntityAuthorship {
    let definition = AuthoredDefinitionId::new("fn_main");
    let site = AuthoredSite {
        definition: definition.clone(),
        function: AuthoredFunctionName::new("main"),
        source: AuthoredSourceId::new("main.veac"),
        span: AuthoredSpan { start: 0, end: 4 },
    };
    EntityAuthorship {
        logical_path: path
            .iter()
            .map(|segment| LogicalPathSegment::new(*segment))
            .collect(),
        events: vec![AuthorshipEvent {
            kind: AuthorshipEventKind::Constructor,
            operation: DomainOpcode(0x1001),
            origin: site,
            definition: AuthoredDefinition {
                identity: definition,
                kind: AuthoredDefinitionKind::Function,
                name: AuthoredFunctionName::new("main"),
                source: AuthoredSourceId::new("main.veac"),
                span: AuthoredSpan { start: 0, end: 4 },
            },
            call_stack: Vec::new(),
            iterations: Vec::new(),
        }],
    }
}
