#[path = "annotation_tests/mapping.rs"]
mod mapping;
#[path = "annotation_tests/rejection.rs"]
mod rejection;
#[path = "annotation_tests/tamper.rs"]
mod tamper;

use crate::*;
use veac_ir::*;

use super::support::*;

struct AnalysisCase {
    project: ProjectEnvelope,
    request: ProviderRequestEnvelope,
    response: ProviderResponseEnvelope,
    context: ApplicationContext,
    proposal: ProviderEditProposal,
}

fn analysis_case(capability: Capability) -> AnalysisCase {
    let project = project();
    let (request, response) = exchange(capability);
    let context = analysis_context(capability);
    let proposal = propose_edit(&project, &request, &response, &context).unwrap();
    AnalysisCase {
        project,
        request,
        response,
        context,
        proposal,
    }
}

fn application(value: &ApplicationContext) -> &AnnotationApplication {
    let ApplicationContext::AnalysisAnnotations(value) = value else {
        panic!("analysis annotation context")
    };
    value
}

fn application_mut(value: &mut ApplicationContext) -> &mut AnnotationApplication {
    let ApplicationContext::AnalysisAnnotations(value) = value else {
        panic!("analysis annotation context")
    };
    value
}

fn annotation(value: &ProviderEditProposal, index: usize) -> &Annotation {
    let EditOperation::InsertAnnotation { annotation } = &value.batch.operations[index] else {
        panic!("insert annotation operation")
    };
    annotation
}

fn annotation_mut(value: &mut ProviderEditProposal, index: usize) -> &mut Annotation {
    let EditOperation::InsertAnnotation { annotation } = &mut value.batch.operations[index] else {
        panic!("insert annotation operation")
    };
    annotation
}

fn assert_evidence(value: &AnalysisCase, index: usize, kind: AnalysisEvidenceKind, result: u32) {
    assert!(matches!(
        &value.proposal.evidence[index],
        ProposalEvidence::AnalysisAnnotation {
            kind: actual,
            result_index,
            ..
        } if *actual == kind && *result_index == result
    ));
}

fn assert_common(value: &AnalysisCase) {
    assert_eq!(
        value.proposal.request_hash,
        request_hash(&value.request).unwrap()
    );
    assert_eq!(
        value.proposal.response_hash,
        response_hash(&value.response).unwrap()
    );
    assert_eq!(value.proposal.source_output, value.response.output);
    assert_eq!(value.proposal.source_request.as_ref(), &value.request);
    assert_eq!(value.proposal.application_context.as_ref(), &value.context);
    assert_eq!(
        value.proposal.project_timebase,
        value.project.project.timebase
    );
    let expected_target = &application(&value.context).target;
    let producer = format!(
        "{}/{}@{}",
        value.request.provider.provider,
        value.request.provider.model,
        value.request.provider.model_version
    );
    for (index, (operation, evidence)) in value
        .proposal
        .batch
        .operations
        .iter()
        .zip(&value.proposal.evidence)
        .enumerate()
    {
        let inserted = annotation(&value.proposal, index);
        assert_eq!(&inserted.target, expected_target);
        let provenance = inserted.provenance.as_ref().unwrap();
        assert_eq!(provenance.producer, producer);
        assert_eq!(provenance.request_sha256, value.proposal.request_hash.value);
        assert_eq!(
            provenance.response_sha256,
            value.proposal.response_hash.value
        );
        let ProposalEvidence::AnalysisAnnotation {
            operation: binding,
            annotation_id,
            ..
        } = evidence
        else {
            panic!("analysis annotation evidence")
        };
        assert_eq!(binding.operation_index as usize, index);
        assert!(binding.matches(operation));
        assert_eq!(annotation_id, &inserted.id);
    }
    let EditOutcome::Applied {
        project: applied, ..
    } = veac_ir::apply_edit_batch(&value.project, &value.proposal.batch)
    else {
        panic!("proposal applies")
    };
    let mut expected: Vec<_> = (0..value.proposal.batch.operations.len())
        .map(|index| annotation(&value.proposal, index).clone())
        .collect();
    expected.sort_by(|left, right| left.id.cmp(&right.id));
    assert_eq!(applied.project.annotations, expected);
}

fn assert_invalid_proposal(value: &ProviderEditProposal) {
    assert_eq!(
        canonical_edit_proposal_bytes(value).unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
}
