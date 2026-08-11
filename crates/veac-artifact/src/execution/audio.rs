use veac_ir::AudioStreamInfo;

use crate::ProxyAudioSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundAudioFacts {
    sample_rate: u32,
    channels: u8,
}

impl BoundAudioFacts {
    pub(super) fn original(info: &AudioStreamInfo) -> Self {
        Self {
            sample_rate: info.sample_rate,
            channels: info.channels,
        }
    }

    pub(super) fn proxy(spec: &ProxyAudioSpec) -> Self {
        Self {
            sample_rate: spec.sample_rate,
            channels: spec.channels,
        }
    }

    pub fn sample_rate(self) -> u32 {
        self.sample_rate
    }

    pub fn channels(self) -> u8 {
        self.channels
    }
}
