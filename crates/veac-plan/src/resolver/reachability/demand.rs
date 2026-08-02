mod audio;

use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{
    AdaptivePackage, CaptionOutput, CaptionSidecarFormat, DeliverableKind, RenderConfig, TimeRange,
    TrackId,
};

use audio::{AudioControls, AudioDemand};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::resolver) enum TextDemand {
    #[default]
    None,
    Plain,
    Styled,
}

impl TextDemand {
    pub(super) fn merge(&mut self, other: Self) {
        *self = (*self).max(other);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SequenceDemand {
    pub visual: bool,
    pub burn_captions: bool,
    audio: AudioDemand,
    pub sidecars: BTreeMap<TrackId, TextDemand>,
    pub visual_controls: BTreeSet<veac_ir::ItemId>,
    audio_controls: AudioControls,
}

impl SequenceDemand {
    pub(super) fn root(config: &RenderConfig) -> Self {
        let mut value = Self::default();
        for deliverable in &config.deliverables {
            match &deliverable.kind {
                DeliverableKind::Video(settings) => {
                    value.visual = true;
                    if settings.audio.is_some() {
                        value.audio.set_master();
                    }
                }
                DeliverableKind::ImageSequence(_) | DeliverableKind::Scope(_) => {
                    value.visual = true;
                }
                DeliverableKind::AudioStem(settings) => value.audio.add(&settings.source),
                DeliverableKind::AudioFile(settings) => value.audio.add(&settings.source),
                DeliverableKind::AnimatedImage(_) | DeliverableKind::StillImage(_) => {
                    value.visual = true;
                }
                DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) => {
                    value.visual = true;
                    if let Some(audio) = &settings.audio {
                        value.audio.add(&audio.source);
                    }
                }
                DeliverableKind::CaptionSidecar(settings) => {
                    let demand = if settings.format == CaptionSidecarFormat::Ass {
                        TextDemand::Styled
                    } else {
                        TextDemand::Plain
                    };
                    for track_id in &settings.track_ids {
                        value
                            .sidecars
                            .entry(track_id.clone())
                            .or_default()
                            .merge(demand);
                    }
                }
            }
        }
        value.burn_captions = value.visual
            && config
                .raster
                .as_ref()
                .is_some_and(|raster| raster.captions == CaptionOutput::BurnIn);
        value
    }

    pub(super) fn nested(
        visual: bool,
        audio_output: bool,
        audio_control_ranges: Vec<TimeRange>,
        burn_captions: bool,
    ) -> Self {
        Self {
            visual,
            burn_captions: visual && burn_captions,
            audio: if audio_output {
                AudioDemand::master()
            } else {
                AudioDemand::default()
            },
            audio_controls: AudioControls::all_tracks(audio_control_ranges),
            ..Self::default()
        }
    }

    pub(super) fn output_audio(&self, track: &veac_ir::Track) -> bool {
        self.audio.selects(track)
    }

    pub(super) fn control_windows(&self, track_id: &TrackId, range: TimeRange) -> Vec<TimeRange> {
        self.audio_controls.windows(track_id, range)
    }

    pub(super) fn add_audio_control(&mut self, track_id: &TrackId, range: TimeRange) {
        self.audio_controls.add(track_id, range);
    }

    pub(super) fn merge(&mut self, other: &Self) -> bool {
        let mut changed = self.audio.merge(&other.audio);
        changed |= !self.visual && other.visual;
        changed |= !self.burn_captions && other.burn_captions;
        self.visual |= other.visual;
        self.burn_captions |= other.burn_captions;
        changed |= extend_map(&mut self.sidecars, &other.sidecars);
        changed |= extend_set(&mut self.visual_controls, &other.visual_controls);
        changed |= self.audio_controls.merge(&other.audio_controls);
        changed
    }
}

fn extend_map(
    target: &mut BTreeMap<TrackId, TextDemand>,
    source: &BTreeMap<TrackId, TextDemand>,
) -> bool {
    let before = target.clone();
    for (id, demand) in source {
        target.entry(id.clone()).or_default().merge(*demand);
    }
    *target != before
}

fn extend_set<T: Ord + Clone>(target: &mut BTreeSet<T>, source: &BTreeSet<T>) -> bool {
    let before = target.len();
    target.extend(source.iter().cloned());
    target.len() != before
}
