use std::path::Path;
use std::time::{Duration, Instant};

use veac_artifact::{artifact_key, ArtifactStore, MediaArtifactRequest};

use super::{
    cache, command, contract, launch_error, output, postflight, preflight, process, source, tool,
    GeneratedArtifact, MediaWorkflow, WorkflowResult,
};

struct RenderJob<'a> {
    store: &'a ArtifactStore,
    input: &'a Path,
    request: &'a MediaArtifactRequest,
    descriptor: &'a veac_artifact::ArtifactDescriptor,
    source: &'a source::SourceSnapshot,
    deadline: Instant,
}

impl MediaWorkflow {
    pub fn derive(
        &self,
        store: &ArtifactStore,
        input: &Path,
        request: &MediaArtifactRequest,
    ) -> WorkflowResult<GeneratedArtifact> {
        self.derive_while(store, input, request, || true)
    }

    pub fn derive_while(
        &self,
        store: &ArtifactStore,
        input: &Path,
        request: &MediaArtifactRequest,
        mut guard: impl FnMut() -> bool,
    ) -> WorkflowResult<GeneratedArtifact> {
        let deadline =
            Instant::now() + Duration::from_secs(self.limits.max_derivation_wall_seconds);
        active(&mut guard)?;
        contract(request.validate_with_limits(self.limits))?;
        let descriptor = contract(request.descriptor())?;
        let source = source::SourceSnapshot::capture(
            input,
            &request.source_identity,
            self.limits.max_source_bytes,
            deadline,
        )?;
        active(&mut guard)?;
        tool::verify(&self.ffmpeg, &request.producer, deadline)?;
        let key = contract(artifact_key(&descriptor))?;
        preflight::validate(&self.ffprobe, source.path(), request, self.limits, deadline)?;
        active(&mut guard)?;
        if let Some(record) = cache::load(
            store,
            &key,
            &descriptor,
            &self.ffprobe,
            &request.spec,
            self.limits,
            deadline,
        )? {
            active(&mut guard)?;
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
        self.render(
            RenderJob {
                store,
                input,
                request,
                descriptor: &descriptor,
                source: &source,
                deadline,
            },
            &mut guard,
        )
    }

    fn render(
        &self,
        job: RenderJob<'_>,
        guard: &mut impl FnMut() -> bool,
    ) -> WorkflowResult<GeneratedArtifact> {
        let RenderJob {
            store,
            input,
            request,
            descriptor,
            source,
            deadline,
        } = job;
        let staging = tempfile::tempdir()?;
        let output = staging
            .path()
            .join(format!("artifact.{}", request.spec.extension()));
        let arguments = command::arguments(source.path(), &output, &request.spec, self.limits)?;
        let executable = self.ffmpeg.launch_until(deadline).map_err(launch_error)?;
        process::run_while(
            executable.path(),
            &arguments,
            &output,
            self.limits,
            deadline,
            &mut *guard,
        )?;
        active(&mut *guard)?;
        let rendered =
            output::OutputProof::capture(&output, self.limits.max_payload_bytes, deadline)?;
        postflight::validate(&self.ffprobe, &output, &request.spec, deadline)?;
        active(&mut *guard)?;
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

fn active(guard: &mut impl FnMut() -> bool) -> WorkflowResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(super::WorkflowError::new(
            super::WorkflowErrorKind::ResourceLimit,
            "media artifact derivation was cancelled",
        ))
    }
}
