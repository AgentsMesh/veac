use veac_ir::{MediaIdentity, StreamSelection};

use super::InputBinding;
use crate::{BoundAudioFacts, BoundResource, BoundStream, BoundVideoFacts, MediaRole};

impl InputBinding {
    pub fn resource(&self) -> Option<&BoundResource> {
        self.resource.as_ref()
    }

    pub fn video(&self) -> Option<&BoundStream> {
        self.video.as_ref()
    }

    pub fn video_facts(&self) -> Option<BoundVideoFacts> {
        self.video_facts
    }

    pub fn audio(&self) -> Option<&BoundStream> {
        self.audio.as_ref()
    }

    pub fn audio_facts(&self) -> Option<BoundAudioFacts> {
        self.audio_facts
    }

    pub fn stream(&self, role: MediaRole) -> Option<&BoundStream> {
        match role {
            MediaRole::Video => self.video(),
            MediaRole::Audio => self.audio(),
        }
    }

    pub fn source_identity(&self) -> Option<&MediaIdentity> {
        self.source_identity.as_ref()
    }

    pub fn source_stream(&self, role: MediaRole) -> Option<StreamSelection> {
        match role {
            MediaRole::Video => self.source_video,
            MediaRole::Audio => self.source_audio,
        }
    }
}
