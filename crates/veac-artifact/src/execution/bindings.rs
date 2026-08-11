use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use veac_ir::{DeliverableId, HashAlgorithm, MediaIdentity, StreamSelection};
use veac_plan::{PlanInputId, ResolvedInput, ResolvedRenderPlan};

use super::{
    BoundAudioFacts, BoundResource, BoundStream, BoundVideoFacts, FullRenderSegmentBinding,
    MediaRole, SourceClock,
};
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, VerifiedArtifact};

mod contract;
#[path = "bindings/input.rs"]
mod input;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionBindings {
    pub(super) inputs: BTreeMap<PlanInputId, InputBinding>,
    outputs: BTreeMap<DeliverableId, PathBuf>,
    pub(super) full_segment: Option<FullRenderSegmentBinding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InputBinding {
    resource: Option<BoundResource>,
    video: Option<BoundStream>,
    video_facts: Option<BoundVideoFacts>,
    audio: Option<BoundStream>,
    audio_facts: Option<BoundAudioFacts>,
    source_identity: Option<MediaIdentity>,
    source_video: Option<StreamSelection>,
    source_audio: Option<StreamSelection>,
}

impl ExecutionBindings {
    pub fn from_originals(
        plan: &ResolvedRenderPlan,
        paths: &BTreeMap<PlanInputId, PathBuf>,
    ) -> ArtifactResult<Self> {
        let mut value = Self::default();
        for input in &plan.inputs {
            let path = paths.get(&input.id).ok_or_else(|| {
                ArtifactError::new(
                    ArtifactErrorKind::MissingBinding,
                    format!("missing execution binding for {}", input.id),
                )
            })?;
            value.bind_original(input, path.clone())?;
        }
        if paths
            .keys()
            .any(|id| !plan.inputs.iter().any(|input| input.id == *id))
        {
            return invalid("execution bindings contain an input outside the render plan");
        }
        Ok(value)
    }

    pub fn bind_original(&mut self, input: &ResolvedInput, path: PathBuf) -> ArtifactResult<()> {
        validate_path(&path)?;
        let resource = BoundResource::original(path.clone(), input.observed_identity.clone());
        let timescale = input
            .probe
            .as_ref()
            .and_then(|probe| probe.container_duration)
            .map(|time| time.timescale)
            .unwrap_or(1);
        let clock = SourceClock::identity(timescale)?;
        let binding = InputBinding {
            resource: Some(resource.clone()),
            video: input
                .video
                .as_ref()
                .map(|stream| BoundStream::new(resource.clone(), stream.selection, clock)),
            video_facts: input
                .video
                .as_ref()
                .map(|stream| BoundVideoFacts::original(&stream.info)),
            audio: input
                .audio
                .as_ref()
                .map(|stream| BoundStream::new(resource.clone(), stream.selection, clock)),
            audio_facts: input
                .audio
                .as_ref()
                .map(|stream| BoundAudioFacts::original(&stream.info)),
            source_identity: Some(input.observed_identity.clone()),
            source_video: input.video.as_ref().map(|stream| stream.selection),
            source_audio: input.audio.as_ref().map(|stream| stream.selection),
        };
        self.inputs.insert(input.id.clone(), binding);
        Ok(())
    }

    pub fn bind_verified_proxy(
        &mut self,
        input: &ResolvedInput,
        role: MediaRole,
        artifact: &VerifiedArtifact,
    ) -> ArtifactResult<()> {
        let (physical_stream, clock, video_facts, audio_facts) =
            super::proxy::binding(input, role, artifact)?;
        let identity = MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: artifact.record().content.value.clone(),
        };
        let resource = BoundResource::artifact(
            artifact.payload_path().to_owned(),
            identity,
            artifact.descriptor(),
            artifact.record(),
        );
        let stream = BoundStream::new(resource, physical_stream, clock);
        let binding = self.inputs.entry(input.id.clone()).or_default();
        binding.record_source(input, role)?;
        match role {
            MediaRole::Video if input.video.is_some() => {
                binding.video = Some(stream);
                binding.video_facts = video_facts;
            }
            MediaRole::Audio if input.audio.is_some() => {
                binding.audio = Some(stream);
                binding.audio_facts = audio_facts;
            }
            _ => return invalid("proxy role is not present on the resolved input"),
        }
        Ok(())
    }

    pub fn bind_output(&mut self, id: DeliverableId, path: PathBuf) -> ArtifactResult<()> {
        validate_path(&path)?;
        self.outputs.insert(id, path);
        Ok(())
    }

    pub fn input(&self, id: &PlanInputId) -> Option<&InputBinding> {
        self.inputs.get(id)
    }

    pub fn inputs(&self) -> &BTreeMap<PlanInputId, InputBinding> {
        &self.inputs
    }

    pub fn output(&self, id: &DeliverableId) -> Option<&Path> {
        self.outputs.get(id).map(PathBuf::as_path)
    }

    pub fn outputs(&self) -> &BTreeMap<DeliverableId, PathBuf> {
        &self.outputs
    }
}

fn validate_path(path: &Path) -> ArtifactResult<()> {
    if path.as_os_str().is_empty() {
        invalid("execution binding path cannot be empty")
    } else {
        Ok(())
    }
}

pub(super) fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}
