use std::collections::BTreeSet;

use veac_ir::{
    ClipSource, FontRef, MaterialId, ProjectEnvelope, RenderConfigId, SequenceId, TrackKind,
};

use crate::error::{CliError, CliResult};

pub(crate) fn collect(
    envelope: &ProjectEnvelope,
    config_id: &RenderConfigId,
) -> CliResult<BTreeSet<MaterialId>> {
    let Some(config) = envelope
        .project
        .render_configs
        .iter()
        .find(|config| config.id == *config_id)
    else {
        return Err(CliError::new(
            "RENDER_CONFIG_NOT_FOUND",
            format!("render config {config_id} does not exist"),
        ));
    };
    let mut collector = Collector {
        envelope,
        output_audio: config.deliverables.iter().any(|value| match &value.kind {
            veac_ir::DeliverableKind::Video(settings) => settings.audio.is_some(),
            veac_ir::DeliverableKind::AudioStem(_) => true,
            _ => false,
        }),
        visited: BTreeSet::new(),
        materials: BTreeSet::new(),
    };
    collector.sequence(&config.sequence_id);
    Ok(collector.materials)
}

struct Collector<'a> {
    envelope: &'a ProjectEnvelope,
    output_audio: bool,
    visited: BTreeSet<SequenceId>,
    materials: BTreeSet<MaterialId>,
}

impl Collector<'_> {
    fn sequence(&mut self, sequence_id: &SequenceId) {
        if !self.visited.insert(sequence_id.clone()) {
            return;
        }
        let Some(sequence) = self
            .envelope
            .project
            .sequences
            .iter()
            .find(|sequence| sequence.id == *sequence_id)
        else {
            return;
        };
        let has_solo = sequence
            .tracks
            .iter()
            .any(|track| track.state.enabled && track.state.solo);
        for track in &sequence.tracks {
            let active = track.state.enabled && (!has_solo || track.state.solo);
            let visual = active && track.kind != TrackKind::Audio;
            let audio = active
                && self.output_audio
                && !track.state.muted
                && matches!(track.kind, TrackKind::Video | TrackKind::Audio);
            if !visual && !audio {
                continue;
            }
            for clip in track.clips.iter().filter(|clip| clip.enabled) {
                let clip_audio = audio && (track.kind == TrackKind::Audio || clip.audio.is_some());
                self.clip(&clip.source, visual, clip_audio);
                if visual {
                    self.color_luts(clip);
                }
            }
        }
    }

    fn clip(&mut self, source: &ClipSource, visual: bool, audio: bool) {
        match source {
            ClipSource::Media { material_id } if visual || audio => {
                self.materials.insert(material_id.clone());
            }
            ClipSource::FreezeFrame { material_id, .. } => {
                self.materials.insert(material_id.clone());
            }
            ClipSource::Sequence { sequence_id } => self.sequence(sequence_id),
            ClipSource::Multicam { group_id, .. } if visual || audio => {
                if let Some(group) = self
                    .envelope
                    .project
                    .multicam_groups
                    .iter()
                    .find(|group| group.id == *group_id)
                {
                    self.materials
                        .extend(group.angles.iter().map(|angle| angle.material_id.clone()));
                }
            }
            ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => {
                for font in style.font_refs() {
                    if let FontRef::Material { material_id } = font {
                        self.materials.insert(material_id.clone());
                    }
                }
            }
            _ => {}
        }
    }

    fn color_luts(&mut self, clip: &veac_ir::Clip) {
        let stages = clip
            .visual
            .as_ref()
            .and_then(|visual| visual.color_pipeline.as_ref())
            .into_iter()
            .flat_map(|pipeline| &pipeline.stages);
        for stage in stages {
            if let veac_ir::ColorStage::Lut { application } = stage {
                self.materials.insert(application.material_id.clone());
            }
        }
    }
}
