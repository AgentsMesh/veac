mod requirements;
mod video;

use std::path::PathBuf;

use veac_artifact::{BoundResource, ContentDigest, ExecutionBindings};
use veac_plan::canonical::{Deliverable, DeliverableKind};
use veac_plan::ResolvedRenderPlan;

use super::{
    delivery_audio, delivery_caption, delivery_image, delivery_scope, BackendAction, BackendBundle,
    BackendOutput, BackendPhase, BackendProduct, BackendTask, CodegenErrors,
};

pub fn emit_all(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
) -> Result<BackendBundle, CodegenErrors> {
    super::preflight::validate(plan)?;
    let plan_identity = plan_identity(plan)?;
    let protected_resources = protected_resources(plan, bindings)?;
    let mut tasks = Vec::new();
    for deliverable in &plan.output.deliverables {
        match &deliverable.kind {
            DeliverableKind::Video(settings) => {
                video::emit(plan, bindings, deliverable, settings, &mut tasks)?;
            }
            DeliverableKind::ImageSequence(settings) => {
                tasks.push(BackendTask {
                    deliverable_id: deliverable.id.clone(),
                    phase: BackendPhase::Single,
                    product: BackendProduct::ImageSequence,
                    output: BackendOutput::ImageSequence {
                        pattern: bound(deliverable, bindings)?,
                    },
                    action: BackendAction::Ffmpeg(delivery_image::command(
                        plan,
                        bindings,
                        deliverable,
                        settings,
                    )?),
                });
            }
            DeliverableKind::CaptionSidecar(settings) => tasks.push(delivery_caption::task(
                plan,
                bindings,
                deliverable,
                settings,
            )?),
            DeliverableKind::AudioStem(settings) => {
                tasks.push(delivery_audio::task(plan, bindings, deliverable, settings)?)
            }
            DeliverableKind::Scope(settings) => {
                tasks.push(delivery_scope::task(plan, bindings, deliverable, settings)?)
            }
        }
    }
    let requirements = requirements::collect(plan, bindings, &tasks)?;
    Ok(BackendBundle::new(
        plan_identity,
        bindings.substitution_proof(),
        protected_resources,
        requirements,
        tasks,
    ))
}

fn protected_resources(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
) -> Result<Vec<super::BackendResource>, CodegenErrors> {
    let mut resources = Vec::<BoundResource>::new();
    for deliverable in &plan.output.deliverables {
        if matches!(deliverable.kind, DeliverableKind::Video(_)) {
            if let Some(segment) = bindings.full_render_segment() {
                resources.push(segment.resource().clone());
                continue;
            }
        }
        let usage = super::input::usage::required(plan, deliverable);
        super::preflight::validate_resources(plan, bindings, usage.resources())?;
        let routes = super::input::resolve(plan, bindings, deliverable)?;
        resources.extend_from_slice(routes.resources());
    }
    super::resource::protected(&resources)
}

fn plan_identity(plan: &ResolvedRenderPlan) -> Result<ContentDigest, CodegenErrors> {
    match veac_plan::canonical_plan_bytes(plan) {
        Ok(bytes) => Ok(ContentDigest::sha256(bytes)),
        Err(error) => Err(CodegenErrors::one(super::error::diagnostic(
            super::CodegenErrorKind::InvalidPlan,
            "PLAN_IDENTITY_FAILED",
            None,
            error.to_string(),
        ))),
    }
}

fn bound(
    deliverable: &Deliverable,
    bindings: &ExecutionBindings,
) -> Result<PathBuf, CodegenErrors> {
    super::output::bound_path(deliverable, bindings)
}
