use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_plan::ResolvedRenderPlan;

use super::{BoundResource, ExecutionBindings};
use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, FullRenderSegmentContract,
    VerifiedArtifact,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullRenderSegmentBinding {
    resource: BoundResource,
    contract: FullRenderSegmentContract,
}

impl ExecutionBindings {
    pub fn bind_verified_render_segment(
        &mut self,
        plan: &ResolvedRenderPlan,
        contract: &FullRenderSegmentContract,
        artifact: &VerifiedArtifact,
    ) -> ArtifactResult<()> {
        let expected = FullRenderSegmentContract::new(
            plan,
            self.input_substitution_proof(),
            contract.descriptor().producer.clone(),
        )?;
        if expected != *contract || artifact.descriptor() != contract.descriptor() {
            return invalid("render segment does not exactly match this execution contract");
        }
        let identity = MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: artifact.record().content.value.clone(),
        };
        self.full_segment = Some(FullRenderSegmentBinding {
            resource: BoundResource::artifact(
                artifact.payload_path().to_owned(),
                identity,
                artifact.descriptor(),
                artifact.record(),
            ),
            contract: contract.clone(),
        });
        Ok(())
    }

    pub fn full_render_segment(&self) -> Option<&FullRenderSegmentBinding> {
        self.full_segment.as_ref()
    }

    pub fn input_substitution_proof(&self) -> ContentDigest {
        super::proof::execution(self.inputs.iter(), None)
    }

    pub fn substitution_proof(&self) -> ContentDigest {
        super::proof::execution(
            self.inputs.iter(),
            self.full_segment.as_ref().map(|value| &value.resource),
        )
    }
}

impl FullRenderSegmentBinding {
    pub fn resource(&self) -> &BoundResource {
        &self.resource
    }

    pub fn contract(&self) -> &FullRenderSegmentContract {
        &self.contract
    }
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}
