use super::*;

#[test]
fn analysis_context_and_proposal_schema_round_trip_strictly() {
    for capability in [
        Capability::LanguageDetection,
        Capability::SceneDetection,
        Capability::BeatDetection,
        Capability::SilenceDetection,
        Capability::FillerDetection,
        Capability::HighlightDetection,
    ] {
        let context = analysis_context(capability);
        let bytes = serde_json_canonicalizer::to_vec(&context).unwrap();
        let decoded: ApplicationContext = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, context);
    }
    let mut value = serde_json::to_value(analysis_context(Capability::BeatDetection)).unwrap();
    value["parameters"]["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ApplicationContext>(value).is_err());

    let schema = serde_json::to_value(schemars::schema_for!(ApplicationContext)).unwrap();
    assert!(schema.to_string().contains("analysis_annotations"));
    let proposal = analysis_case(Capability::HighlightDetection).proposal;
    let bytes = canonical_edit_proposal_bytes(&proposal).unwrap();
    let decoded: ProviderEditProposal = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(canonical_edit_proposal_bytes(&decoded).unwrap(), bytes);
    assert!(provider_edit_proposal_json_schema()
        .unwrap()
        .to_string()
        .contains("analysis_annotation"));
}

#[test]
fn proposal_validation_rejects_annotation_operation_tampering() {
    let original = analysis_case(Capability::BeatDetection).proposal;

    let mut payload = original.clone();
    let AnnotationPayload::Beat { confidence, .. } = &mut annotation_mut(&mut payload, 0).payload
    else {
        panic!("beat payload")
    };
    *confidence = 0.8;
    rebind(&mut payload, 0);
    assert_invalid_proposal(&payload);

    let mut id = original.clone();
    annotation_mut(&mut id, 0).id = AnnotationId::new("ann_tampered").unwrap();
    rebind(&mut id, 0);
    assert_invalid_proposal(&id);

    let mut target = original.clone();
    annotation_mut(&mut target, 0).target = AnnotationTarget::Project;
    rebind(&mut target, 0);
    assert_invalid_proposal(&target);

    let mut span = original.clone();
    annotation_mut(&mut span, 0).span = AnnotationSpan::Point { at: time(1) };
    rebind(&mut span, 0);
    assert_invalid_proposal(&span);

    let mut provenance = original;
    annotation_mut(&mut provenance, 0)
        .provenance
        .as_mut()
        .unwrap()
        .producer = "tampered/provider@1".into();
    rebind(&mut provenance, 0);
    assert_invalid_proposal(&provenance);
}

#[test]
fn proposal_validation_rejects_rebound_provenance_digests() {
    let original = analysis_case(Capability::SceneDetection).proposal;
    let mut request = original.clone();
    annotation_mut(&mut request, 0)
        .provenance
        .as_mut()
        .unwrap()
        .request_sha256 = "0".repeat(64);
    rebind(&mut request, 0);
    assert_invalid_proposal(&request);

    let mut response = original;
    annotation_mut(&mut response, 0)
        .provenance
        .as_mut()
        .unwrap()
        .response_sha256 = "0".repeat(64);
    rebind(&mut response, 0);
    assert_invalid_proposal(&response);
}

#[test]
fn proposal_validation_rejects_analysis_evidence_tampering() {
    let original = analysis_case(Capability::BeatDetection).proposal;

    let mut kind = original.clone();
    *evidence_mut(&mut kind).1 = AnalysisEvidenceKind::Language;
    assert_invalid_proposal(&kind);

    let mut index = original.clone();
    *evidence_mut(&mut index).2 = 99;
    assert_invalid_proposal(&index);

    let mut id = original.clone();
    *evidence_mut(&mut id).3 = AnnotationId::new("ann_tampered").unwrap();
    assert_invalid_proposal(&id);

    let mut hash = original;
    evidence_mut(&mut hash).0.operation_hash.value = "0".repeat(64);
    assert_invalid_proposal(&hash);
}

#[test]
fn proposal_validation_rejects_tampered_source_output() {
    let mut proposal = analysis_case(Capability::BeatDetection).proposal;
    let ProviderOutput::BeatDetection(value) = &mut proposal.source_output else {
        panic!("beat output")
    };
    value.beats[0].confidence = 0.8;
    assert_invalid_proposal(&proposal);
}

fn rebind(value: &mut ProviderEditProposal, index: usize) {
    let operation = OperationBinding::new(index as u32, &value.batch.operations[index]).unwrap();
    evidence_mut_at(value, index).0.clone_from(&operation);
}

fn evidence_mut(
    value: &mut ProviderEditProposal,
) -> (
    &mut OperationBinding,
    &mut AnalysisEvidenceKind,
    &mut u32,
    &mut AnnotationId,
) {
    evidence_mut_at(value, 0)
}

fn evidence_mut_at(
    value: &mut ProviderEditProposal,
    index: usize,
) -> (
    &mut OperationBinding,
    &mut AnalysisEvidenceKind,
    &mut u32,
    &mut AnnotationId,
) {
    let ProposalEvidence::AnalysisAnnotation {
        operation,
        kind,
        result_index,
        annotation_id,
    } = &mut value.evidence[index]
    else {
        panic!("analysis evidence")
    };
    (operation, kind, result_index, annotation_id)
}
