use veac_plan::ResolvedInput;

use super::{invalid, InputBinding};
use crate::{ArtifactResult, MediaRole};

impl InputBinding {
    pub(super) fn record_source(
        &mut self,
        input: &ResolvedInput,
        role: MediaRole,
    ) -> ArtifactResult<()> {
        if self
            .source_identity
            .as_ref()
            .is_some_and(|identity| identity != &input.observed_identity)
        {
            return invalid("split input bindings disagree on their logical source identity");
        }
        self.source_identity = Some(input.observed_identity.clone());
        let expected = match role {
            MediaRole::Video => input.video.as_ref().map(|value| value.selection),
            MediaRole::Audio => input.audio.as_ref().map(|value| value.selection),
        }
        .ok_or_else(|| {
            crate::ArtifactError::new(
                crate::ArtifactErrorKind::InvalidContract,
                "proxy role is absent from its logical source",
            )
        })?;
        let slot = match role {
            MediaRole::Video => &mut self.source_video,
            MediaRole::Audio => &mut self.source_audio,
        };
        if slot.is_some_and(|selection| selection != expected) {
            return invalid("split input bindings disagree on their logical source stream");
        }
        *slot = Some(expected);
        Ok(())
    }
}

#[cfg(test)]
#[path = "contract/tests.rs"]
mod tests;
