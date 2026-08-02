use crate::test_support::output_for;
use crate::*;

use super::support::*;

#[test]
fn tied_language_scores_require_a_stable_language_order() {
    let result = LanguageDetectionResult {
        languages: vec![
            LanguageScore {
                language: "zh".into(),
                confidence: 0.9,
            },
            LanguageScore {
                language: "en".into(),
                confidence: 0.9,
            },
        ],
    };
    assert_eq!(
        result.validate().unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
}

#[test]
fn dubbing_rejects_overlapping_rendered_turns() {
    let (mut request, _) = exchange(Capability::Dubbing);
    let ProviderRequest::Dubbing(value) = &mut request.request else {
        unreachable!()
    };
    let mut requested = value.turns[0].clone();
    requested.id = "turn-2".into();
    requested.source_range = crate::test_support::range(20, 10);
    value.turns.push(requested);

    let mut output = output_for(&request);
    let ProviderOutput::Dubbing(result) = &mut output else {
        unreachable!()
    };
    let mut rendered = result.turns[0].clone();
    rendered.id = "turn-2".into();
    rendered.source_range = crate::test_support::range(20, 10);
    rendered.rendered_range = crate::test_support::range(5, 10);
    result.turns.push(rendered);
    let artifact = result.audio.clone();
    let response = ProviderResponseEnvelope::new(&request, output).unwrap();

    let error =
        propose_edit(&project(), &request, &response, &dubbing_context(&artifact)).unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert!(error.to_string().contains("ordered and non-overlapping"));
}

#[test]
fn proposal_source_contract_rejects_request_and_context_tampering() {
    let (request, response) = exchange(Capability::Asr);
    let proposal = propose_edit(&project(), &request, &response, &asr_context()).unwrap();

    let mut source_request = proposal.clone();
    source_request.source_request.provider.model_version = "tampered".into();
    assert_invalid_proposal(&source_request);

    let mut application = proposal.clone();
    header_mut(&mut application.application_context).operation_id =
        veac_ir::OperationId::new("op_other").unwrap();
    assert_invalid_proposal(&application);

    let mut timebase = proposal;
    timebase.project_timebase = 0;
    assert_invalid_proposal(&timebase);
}

fn assert_invalid_proposal(value: &ProviderEditProposal) {
    assert_eq!(
        canonical_edit_proposal_bytes(value).unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
}
