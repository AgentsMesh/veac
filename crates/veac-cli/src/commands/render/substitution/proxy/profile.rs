use veac_ir::{DeliverableKind, RasterSettings};
use veac_plan::ResolvedOutput;

use crate::error::{CliError, CliResult};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct ProxyProfile {
    pub(super) raster: Option<RasterSettings>,
    pub(super) audio: Option<ProxyAudioFormat>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ProxyAudioFormat {
    pub(super) sample_rate: u32,
    pub(super) channels: u8,
}

impl ProxyProfile {
    pub(super) fn is_empty(&self) -> bool {
        self.raster.is_none() && self.audio.is_none()
    }
}

pub(super) fn resolve(output: &ResolvedOutput) -> CliResult<ProxyProfile> {
    let mut audio: Vec<_> = output
        .deliverables
        .iter()
        .filter_map(|value| match &value.kind {
            DeliverableKind::Video(settings) => settings.audio.as_ref(),
            DeliverableKind::AudioStem(settings) => Some(&settings.audio),
            _ => None,
        })
        .map(|value| ProxyAudioFormat {
            sample_rate: value.sample_rate,
            channels: value.channels,
        })
        .collect();
    audio.sort_by_key(|value| (value.sample_rate, value.channels));
    audio.dedup();
    if audio.len() > 1 {
        return Err(CliError::new(
            "PROXY_AUDIO_FORMAT_AMBIGUOUS",
            "proxy substitution requires one audio sample-rate/channel format per delivery",
        ));
    }
    Ok(ProxyProfile {
        raster: output.raster.clone(),
        audio: audio.first().copied(),
    })
}
