use veac_ir::{
    EditOperation, MatteRelationParameters, ProjectEnvelope, Relation, RelationEndpoint,
    RelationId, RelationKind, StructureEdit,
};

use crate::{
    OperationBinding, ProposalEvidence, ProviderArtifact, ProviderError, ProviderErrorKind,
    ProviderResult, RetouchApplication, RetouchResult,
};

use super::super::{media, support, visual_media};
use super::BuiltApplication;

pub(super) fn append_optional<'a>(
    project: &ProjectEnvelope,
    result: &'a RetouchResult,
    context: &RetouchApplication,
    sequence: &veac_ir::Sequence,
    record_range: veac_ir::TimeRange,
    built: &mut BuiltApplication,
) -> ProviderResult<Option<&'a ProviderArtifact>> {
    let (application, artifact) = match (&context.matte, result.masks.as_slice()) {
        (None, []) => return Ok(None),
        (Some(application), [artifact]) => (application, artifact),
        _ => {
            return support::unsupported(
                "retouch can apply either no matte or one executable matte",
            )
        }
    };
    if application.insertion.sequence_id != context.sequence_id {
        return support::invalid("retouch matte must share the apply sequence");
    }
    let (track, matte_range, source_start) = visual_media::validate_matte(
        project,
        artifact,
        &application.insertion,
        sequence,
        record_range,
    )?;
    visual_media::append_matte(
        built,
        &application.insertion,
        artifact,
        matte_range,
        source_start,
    )?;
    media::add_track_precondition(built, track);
    Ok(Some(artifact))
}

pub(super) fn append_relation(
    context: &RetouchApplication,
    artifact: &ProviderArtifact,
    built: &mut BuiltApplication,
) -> ProviderResult<()> {
    let matte = context
        .matte
        .as_ref()
        .expect("artifact requires matte context");
    let relation_id = RelationId::new(format!("rel_matte_{}", context.apply_id))
        .map_err(|_| invalid("invalid retouch matte relation ID"))?;
    let operation = EditOperation::EditStructure {
        edit: StructureEdit::InsertRelation {
            relation: Relation {
                id: relation_id,
                sequence_id: context.sequence_id.clone(),
                kind: RelationKind::Matte {
                    producer: RelationEndpoint::item(matte.insertion.clip_id.clone()),
                    consumer: RelationEndpoint::apply(context.apply_id.clone()),
                    parameters: MatteRelationParameters {
                        mode: matte.mode,
                        invert: matte.invert,
                    },
                },
            },
        },
    };
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::AppliedTrackMatte {
        operation: OperationBinding::new(index, &operation)?,
        artifact_key: artifact.record.key.clone(),
        artifact_role: artifact.role.clone(),
        matte_clip_id: matte.insertion.clip_id.clone(),
    });
    built.operations.push(operation);
    Ok(())
}

fn invalid(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}
