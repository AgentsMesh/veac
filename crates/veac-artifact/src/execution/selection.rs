use veac_plan::ResolvedInput;

use super::{ExecutionBindings, MediaRole};
use crate::{ArtifactResult, ProxyBinding};

impl ExecutionBindings {
    pub fn bind_proxy_selection(
        &mut self,
        input: &ResolvedInput,
        selection: &ProxyBinding,
    ) -> ArtifactResult<()> {
        if selection.source_identity != super::proxy::media_digest(&input.observed_identity)? {
            return super::bindings::invalid("proxy selection belongs to another resolved input");
        }
        if let Some(video) = &selection.video {
            self.bind_verified_proxy(input, MediaRole::Video, video)?;
        }
        if let Some(audio) = &selection.audio {
            self.bind_verified_proxy(input, MediaRole::Audio, audio)?;
        }
        Ok(())
    }
}
