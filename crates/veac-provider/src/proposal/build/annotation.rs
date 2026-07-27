use veac_ir::{
    Annotation, AnnotationPayload, AnnotationProvenance, AnnotationSpan, AnnotationTarget,
    EditOperation, Precondition, ProjectEnvelope, RationalTime, TimeRange,
};

use super::support::{self, BuiltApplication};
use crate::{
    AnalysisEvidenceKind, AnnotationApplication, ClipTimeBinding, OperationBinding,
    ProposalEvidence, ProviderError, ProviderErrorKind, ProviderOutput, ProviderResult,
};

mod output;

pub(super) fn build(
    project: &ProjectEnvelope,
    output: &ProviderOutput,
    context: &AnnotationApplication,
    provenance: &AnnotationProvenance,
) -> ProviderResult<BuiltApplication> {
    let timed = !matches!(output, ProviderOutput::LanguageDetection(_));
    if timed != context.time.is_some()
        || (timed
            && matches!(
                context.target,
                AnnotationTarget::Project | AnnotationTarget::MulticamGroup { .. }
            ))
    {
        return support::invalid("analysis annotation time binding does not match its output");
    }
    let mut builder = Builder {
        project,
        context,
        provenance,
        built: BuiltApplication::new(Vec::new(), Vec::new()),
    };
    output::append(&mut builder, output)?;
    builder.finish()
}

pub(super) struct Builder<'a> {
    project: &'a ProjectEnvelope,
    context: &'a AnnotationApplication,
    provenance: &'a AnnotationProvenance,
    built: BuiltApplication,
}

impl Builder<'_> {
    pub(super) fn point(&self, value: RationalTime) -> ProviderResult<AnnotationSpan> {
        let binding = self
            .context
            .time
            .ok_or_else(|| invalid_error("timed analysis annotation requires a time binding"))?;
        let mapped = super::super::time::clip_time(
            value,
            ClipTimeBinding {
                provider_origin: binding.provider_origin,
                clip_local_origin: binding.target_origin,
            },
            self.project.project.timebase,
        )?;
        if mapped.value < 0 {
            return support::invalid("analysis annotation maps before target time zero");
        }
        Ok(AnnotationSpan::Point { at: mapped })
    }

    pub(super) fn range(&self, value: TimeRange) -> ProviderResult<AnnotationSpan> {
        let AnnotationSpan::Point { at } = self.point(value.start)? else {
            unreachable!()
        };
        let duration = super::super::time::convert(value.duration, self.project.project.timebase)?;
        Ok(AnnotationSpan::Range {
            range: TimeRange::new(at, duration).map_err(|error| {
                ProviderError::with_source(
                    ProviderErrorKind::InvalidContract,
                    "analysis annotation range is invalid",
                    error,
                )
            })?,
        })
    }

    pub(super) fn push(
        &mut self,
        role: &str,
        kind: AnalysisEvidenceKind,
        result_index: usize,
        span: AnnotationSpan,
        payload: AnnotationPayload,
    ) -> ProviderResult<()> {
        let id = veac_ir::AnnotationId::new(format!(
            "{}_{}_{:04}",
            self.context.annotation_id_prefix,
            role,
            result_index + 1
        ))
        .map_err(|error| {
            ProviderError::with_source(
                ProviderErrorKind::InvalidContract,
                "analysis annotation ID prefix is invalid",
                error,
            )
        })?;
        if self
            .project
            .project
            .annotations
            .iter()
            .any(|value| value.id == id)
        {
            return support::invalid("analysis annotation ID already exists");
        }
        let operation_index = support::index(self.built.operations.len())?;
        let operation = EditOperation::InsertAnnotation {
            annotation: Box::new(Annotation {
                id: id.clone(),
                target: self.context.target.clone(),
                span,
                payload,
                provenance: Some(self.provenance.clone()),
            }),
        };
        self.built
            .evidence
            .push(ProposalEvidence::AnalysisAnnotation {
                operation: OperationBinding::new(operation_index, &operation)?,
                kind,
                result_index: support::index(result_index)?,
                annotation_id: id,
            });
        self.built.operations.push(operation);
        Ok(())
    }

    fn finish(mut self) -> ProviderResult<BuiltApplication> {
        if self.built.operations.is_empty() {
            return support::unsupported("analysis result contains no annotations to apply");
        }
        if let AnnotationTarget::Clip { clip_id } = &self.context.target {
            support::clip(self.project, clip_id)?;
            self.built.preconditions.push(Precondition::ClipExists {
                clip_id: clip_id.clone(),
            });
        }
        Ok(self.built)
    }
}

fn invalid_error(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}
