use std::path::Path;
use std::time::{Duration, Instant};

use veac_artifact::{artifact_key, ArtifactStore, MediaArtifactRequest, MediaArtifactSpec};

use super::{
    cache, command, contract, launch_error, output, postflight, preflight, process, source, tool,
    unsupported, GeneratedArtifact, MediaWorkflow, WorkflowResult,
};

impl MediaWorkflow {
    pub fn derive(
        &self,
        store: &ArtifactStore,
        input: &Path,
        request: &MediaArtifactRequest,
    ) -> WorkflowResult<GeneratedArtifact> {
        let deadline =
            Instant::now() + Duration::from_secs(self.limits.max_derivation_wall_seconds);
        contract(request.validate_with_limits(self.limits))?;
        if matches!(request.spec, MediaArtifactSpec::Analysis(_)) {
            return unsupported("analysis artifacts require explicit canonical analysis data");
        }
        let descriptor = contract(request.descriptor())?;
        let source = source::SourceSnapshot::capture(
            input,
            &request.source_identity,
            self.limits.max_source_bytes,
            deadline,
        )?;
        tool::verify(&self.ffmpeg, &request.producer, deadline)?;
        let key = contract(artifact_key(&descriptor))?;
        preflight::validate(&self.ffprobe, source.path(), request, self.limits, deadline)?;
        if let Some(record) = cache::load(
            store,
            &key,
            &descriptor,
            &self.ffprobe,
            &request.spec,
            self.limits,
            deadline,
        )? {
            source::verify_until(
                input,
                &request.source_identity,
                self.limits.max_source_bytes,
                deadline,
            )?;
            return Ok(GeneratedArtifact {
                record,
                cache_hit: true,
            });
        }
        self.render(store, input, request, &descriptor, &source, deadline)
    }

    fn render(
        &self,
        store: &ArtifactStore,
        input: &Path,
        request: &MediaArtifactRequest,
        descriptor: &veac_artifact::ArtifactDescriptor,
        source: &source::SourceSnapshot,
        deadline: Instant,
    ) -> WorkflowResult<GeneratedArtifact> {
        let staging = tempfile::tempdir()?;
        let output = staging
            .path()
            .join(format!("artifact.{}", request.spec.extension()));
        let arguments = command::arguments(source.path(), &output, &request.spec, self.limits)?;
        let executable = self.ffmpeg.launch_until(deadline).map_err(launch_error)?;
        process::run(
            executable.path(),
            &arguments,
            &output,
            self.limits,
            deadline,
        )?;
        let rendered =
            output::OutputProof::capture(&output, self.limits.max_payload_bytes, deadline)?;
        postflight::validate(&self.ffprobe, &output, &request.spec, deadline)?;
        rendered.reverify(&output, self.limits.max_payload_bytes, deadline)?;
        source::verify_until(
            input,
            &request.source_identity,
            self.limits.max_source_bytes,
            deadline,
        )?;
        let record = rendered.store(store, descriptor, &output, deadline)?;
        source::verify_until(
            input,
            &request.source_identity,
            self.limits.max_source_bytes,
            deadline,
        )?;
        Ok(GeneratedArtifact {
            record,
            cache_hit: false,
        })
    }
}
