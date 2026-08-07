use veac_ir::{Clip, ClipSource, SequenceId};

use super::{components, error, material::InputUsage, PlanResolver};
use crate::{ResolvedClip, ResolvedClipSource, ResolvedSourceMapping};

use super::reachability::{ClipDemand, TextDemand};

impl PlanResolver<'_> {
    pub(super) fn resolve_clip(
        &mut self,
        sequence_id: &SequenceId,
        clip: &Clip,
        demand: ClipDemand,
        source_order: u32,
    ) -> Option<ResolvedClip> {
        let path = format!("/project/sequences/tracks/clips/{}", clip.id);
        let audio_demand = demand.audio();
        let (source, source_mapping) =
            self.resolve_source(clip, demand.visual, audio_demand, demand.text, &path)?;
        let retain_visual = demand.visual || demand.text == TextDemand::Styled;
        let visual = retain_visual.then(|| {
            let mut value = self.resolve_visual(clip.visual.as_ref());
            value.track_matte = demand
                .visual
                .then(|| self.resolved_matte(sequence_id, &clip.id))
                .flatten();
            value
        });
        let audio = audio_demand.then(|| {
            let mut value = super::defaults::audio(clip.audio.as_ref());
            value.sidechain = demand
                .audio_output
                .then(|| self.resolved_sidechain(sequence_id, &clip.id))
                .flatten();
            value
        });
        Some(ResolvedClip {
            id: clip.id.clone(),
            source_order,
            record_range: clip.record_range,
            source,
            source_mapping,
            visual,
            audio,
            effects: super::effects::resolve_effects(clip, demand.visual, audio_demand),
        })
    }

    fn resolve_source(
        &mut self,
        clip: &Clip,
        visual: bool,
        audio: bool,
        text_demand: TextDemand,
        path: &str,
    ) -> Option<(ResolvedClipSource, Option<ResolvedSourceMapping>)> {
        match &clip.source {
            ClipSource::Media { material_id } => {
                let Some(mapping) = &clip.source_mapping else {
                    self.push_internal(
                        "SOURCE_MAPPING_MISSING",
                        clip.id.to_string(),
                        "validated media clip has no source mapping".to_owned(),
                    );
                    return None;
                };
                let usage = InputUsage {
                    video: visual,
                    audio,
                    font: false,
                };
                let input = self.material_input(material_id, usage)?;
                let mapping = self.resolve_source_mapping(mapping, clip, &input, usage, path)?;
                Some((
                    ResolvedClipSource::Media {
                        input_id: input.id,
                        video_stream: visual
                            .then(|| input.video.as_ref().map(|stream| stream.selection))
                            .flatten(),
                        audio_stream: audio
                            .then(|| input.audio.as_ref().map(|stream| stream.selection))
                            .flatten(),
                    },
                    Some(mapping),
                ))
            }
            ClipSource::FreezeFrame {
                material_id,
                source_time,
            } => {
                let usage = InputUsage {
                    video: true,
                    audio: false,
                    font: false,
                };
                let input = self.material_input(material_id, usage)?;
                self.check_freeze_bounds(&input, *source_time, clip, path);
                let selection = input.video.as_ref().map(|stream| stream.selection)?;
                Some((
                    ResolvedClipSource::FreezeFrame {
                        input_id: input.id,
                        video_stream: selection,
                        source_time: *source_time,
                    },
                    None,
                ))
            }
            ClipSource::Sequence { sequence_id } => {
                self.resolve_sequence(sequence_id);
                let sequence = self.sequences.get(sequence_id)?;
                let duration = sequence.duration;
                if duration.value <= 0 {
                    return None;
                }
                let components = components::sequence(sequence);
                if visual && !components.video {
                    self.push(error::nested_component_missing(
                        path,
                        sequence_id.as_str(),
                        "video",
                    ));
                }
                if audio && !components.audio {
                    self.push(error::nested_component_missing(
                        path,
                        sequence_id.as_str(),
                        "audio",
                    ));
                }
                let mapping = self.resolve_sequence_mapping(
                    clip.source_mapping.as_ref()?,
                    clip,
                    duration,
                    path,
                )?;
                Some((
                    ResolvedClipSource::Sequence {
                        sequence_id: sequence_id.clone(),
                    },
                    Some(mapping),
                ))
            }
            ClipSource::Multicam { group_id, switches } => self
                .resolve_multicam(clip, group_id, switches, visual, audio, path)
                .map(|source| (source, None)),
            ClipSource::Text { text, style } => Some((
                ResolvedClipSource::Text {
                    content: self.resolve_text(text, style, text_demand, path)?,
                },
                None,
            )),
            ClipSource::Caption {
                text,
                speaker,
                style,
                cue,
            } => Some((
                ResolvedClipSource::Caption {
                    content: self.resolve_text(text, style, text_demand, path)?,
                    speaker: speaker.clone(),
                    cue: cue.as_ref().clone(),
                },
                None,
            )),
            ClipSource::Generated { generator } => Some((
                ResolvedClipSource::Generated {
                    generator: generator.clone(),
                },
                None,
            )),
        }
    }
}
