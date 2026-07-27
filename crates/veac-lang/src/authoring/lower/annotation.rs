use crate::authoring::{AnnotationDecl, AnnotationTargetDecl, AnnotationTimingDecl, Identifier};
use veac_ir::{Annotation, AnnotationProvenance, AnnotationSpan, AnnotationTarget};

use super::context::Context;
use super::{annotation_payload, ids, value};

pub(super) fn lower(
    ctx: &mut Context,
    declaration: &AnnotationDecl,
    project_id: &Identifier,
) -> Option<Annotation> {
    Some(Annotation {
        id: ids::annotation(ctx, &declaration.id)?,
        target: target(ctx, &declaration.target, project_id)?,
        span: timing(ctx, &declaration.timing)?,
        payload: annotation_payload::lower(ctx, &declaration.payload)?,
        provenance: Some(AnnotationProvenance {
            producer: declaration.provenance.producer.value.clone(),
            request_sha256: declaration.provenance.request_sha256.value.clone(),
            response_sha256: declaration.provenance.response_sha256.value.clone(),
        }),
    })
}

fn target(
    ctx: &mut Context,
    value: &AnnotationTargetDecl,
    project_id: &Identifier,
) -> Option<AnnotationTarget> {
    Some(match value {
        AnnotationTargetDecl::Project { .. } => {
            let _ = ids::project(ctx, project_id)?;
            AnnotationTarget::Project
        }
        AnnotationTargetDecl::Sequence(id) => AnnotationTarget::Sequence {
            sequence_id: ids::sequence(ctx, id)?,
        },
        AnnotationTargetDecl::Layer(id) => AnnotationTarget::Track {
            track_id: ids::track(ctx, id)?,
        },
        AnnotationTargetDecl::Item(id) => AnnotationTarget::Clip {
            clip_id: ids::item(ctx, id)?,
        },
        AnnotationTargetDecl::Resource(id) => AnnotationTarget::Material {
            material_id: ids::material(ctx, id)?,
        },
        AnnotationTargetDecl::Multicam(id) => AnnotationTarget::MulticamGroup {
            group_id: ids::multicam(ctx, id)?,
        },
    })
}

fn timing(ctx: &mut Context, value: &AnnotationTimingDecl) -> Option<AnnotationSpan> {
    Some(match value {
        AnnotationTimingDecl::Untimed { .. } => AnnotationSpan::Untimed,
        AnnotationTimingDecl::Point { at, .. } => AnnotationSpan::Point {
            at: value::time(ctx, at)?,
        },
        AnnotationTimingDecl::Range { at, duration, .. } => AnnotationSpan::Range {
            range: value::range(ctx, at, duration)?,
        },
    })
}
