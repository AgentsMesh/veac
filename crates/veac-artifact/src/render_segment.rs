use veac_ir::{DeliverableId, DeliverableKind, TimeRange};
use veac_plan::ResolvedRenderPlan;

use crate::{
    artifact_key, ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor, ArtifactError,
    ArtifactErrorKind, ArtifactParameters, ArtifactResult, ArtifactStore, ContentDigest,
    ProducerFingerprint, RenderSegmentFidelity, RenderSegmentParameters, VerifiedArtifact,
};

mod profile;

pub use profile::FullRenderSegmentMediaProfile;

#[derive(Debug, Clone, PartialEq)]
pub struct FullRenderSegmentContract {
    descriptor: ArtifactDescriptor,
    range: TimeRange,
    media_profile: FullRenderSegmentMediaProfile,
}

impl Eq for FullRenderSegmentContract {}

impl FullRenderSegmentContract {
    pub fn new(
        plan: &ResolvedRenderPlan,
        source_clocks: ContentDigest,
        producer: ProducerFingerprint,
    ) -> ArtifactResult<Self> {
        source_clocks.validate()?;
        if plan.output.sequence_id != plan.entry_sequence_id || plan.output.deliverables.len() != 1
        {
            return invalid("full render-segment substitution requires one entry-sequence output");
        }
        let deliverable = &plan.output.deliverables[0];
        let DeliverableKind::Video(_) = &deliverable.kind else {
            return invalid("full render-segment substitution requires one video deliverable");
        };
        let Some(raster) = &plan.output.raster else {
            return invalid("full render-segment substitution requires a delivery raster");
        };
        let sequence = plan
            .sequences
            .iter()
            .find(|value| value.id == plan.entry_sequence_id)
            .ok_or_else(|| {
                ArtifactError::new(
                    ArtifactErrorKind::InvalidContract,
                    "render plan has no resolved entry sequence",
                )
            })?;
        let range = TimeRange::new(
            veac_ir::RationalTime::zero(sequence.duration.timescale).map_err(time_error)?,
            sequence.duration,
        )
        .map_err(time_error)?;
        let plan_hash = veac_plan::plan_hash(plan).map_err(|error| {
            ArtifactError::new(
                ArtifactErrorKind::InvalidContract,
                format!("render plan cannot be hashed for segment substitution: {error}"),
            )
        })?;
        let plan_identity = ContentDigest {
            algorithm: crate::DigestAlgorithm::Sha256,
            value: plan_hash,
        };
        plan_identity.validate()?;
        let media_profile = FullRenderSegmentMediaProfile::new(raster, deliverable);
        let profile = media_profile.digest()?;
        let parameters = ArtifactParameters::RenderSegment(RenderSegmentParameters {
            sequence_id: plan.entry_sequence_id.clone(),
            range,
            deliverable_id: deliverable.id.clone(),
            fidelity: RenderSegmentFidelity::ExactDeliveryMaster,
        });
        let descriptor = ArtifactDescriptor::new(
            producer,
            vec![
                dependency(ArtifactDependencyRole::Plan, plan_identity),
                dependency(ArtifactDependencyRole::Profile, profile),
                dependency(ArtifactDependencyRole::SourceClocks, source_clocks),
            ],
            parameters,
        );
        descriptor.validate()?;
        Ok(Self {
            descriptor,
            range,
            media_profile,
        })
    }

    pub fn descriptor(&self) -> &ArtifactDescriptor {
        &self.descriptor
    }

    pub fn range(&self) -> TimeRange {
        self.range
    }

    pub fn deliverable_id(&self) -> &DeliverableId {
        &self.media_profile.deliverable().id
    }

    pub fn has_audio(&self) -> bool {
        self.media_profile.video().audio.is_some()
    }

    pub fn media_profile(&self) -> &FullRenderSegmentMediaProfile {
        &self.media_profile
    }
}

pub fn select_full_render_segment(
    store: &ArtifactStore,
    contract: &FullRenderSegmentContract,
) -> ArtifactResult<Option<VerifiedArtifact>> {
    select_full_render_segment_while(store, contract, || true)
}

pub fn select_full_render_segment_while(
    store: &ArtifactStore,
    contract: &FullRenderSegmentContract,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<Option<VerifiedArtifact>> {
    active(&mut guard)?;
    let key = artifact_key(contract.descriptor())?;
    store.open_verified_bounded_while(
        &key,
        contract.descriptor(),
        crate::MAX_ARTIFACT_PAYLOAD_BYTES,
        guard,
    )
}

fn active(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "render-segment selection exceeded its caller resource guard",
        ))
    }
}

fn dependency(role: ArtifactDependencyRole, identity: ContentDigest) -> ArtifactDependency {
    ArtifactDependency::new(role, identity)
}

fn time_error(error: veac_ir::TimeError) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::InvalidContract,
        "entry sequence does not define a valid full segment range",
        error,
    )
}

pub(crate) fn serialization_error(error: serde_json::Error) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::Serialization,
        "render-segment identity cannot be serialized",
        error,
    )
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

#[cfg(test)]
#[path = "render_segment/guard_tests.rs"]
mod guard_tests;
