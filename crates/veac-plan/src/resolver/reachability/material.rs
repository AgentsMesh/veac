use std::collections::BTreeMap;

use veac_ir::{
    ApplyOperation, ClipSource, ColorStage, FontRef, MaterialId, ProjectEnvelope, TrackId,
};

use crate::EffectiveTrackState;

use super::{apply, ClipDemand};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct MaterialUse {
    video: bool,
    audio: bool,
    font: bool,
    lut: bool,
}

impl MaterialUse {
    fn merge(&mut self, other: Self) {
        self.video |= other.video;
        self.audio |= other.audio;
        self.font |= other.font;
        self.lut |= other.lut;
    }
}

pub(super) fn collect(
    envelope: &ProjectEnvelope,
    tracks: &BTreeMap<(veac_ir::SequenceId, TrackId), EffectiveTrackState>,
    clips: &BTreeMap<(veac_ir::SequenceId, veac_ir::ItemId), ClipDemand>,
) -> BTreeMap<MaterialId, MaterialUse> {
    let mut output = BTreeMap::new();
    for sequence in &envelope.project.sequences {
        if !tracks.keys().any(|(id, _)| id == &sequence.id) {
            continue;
        }
        for track in &sequence.tracks {
            for clip in &track.clips {
                let Some(demand) = clips.get(&(sequence.id.clone(), clip.id.clone())).copied()
                else {
                    continue;
                };
                source(envelope, &clip.source, demand, &mut output);
                if demand.visual {
                    color(
                        clip.visual
                            .as_ref()
                            .and_then(|value| value.color_pipeline.as_ref()),
                        &mut output,
                    );
                }
            }
        }
        for value in &sequence.applies {
            for stage in value.stages.iter().filter(|stage| {
                apply::stage_active(sequence, value, stage, |id| {
                    clips.get(&(sequence.id.clone(), id.clone())).copied()
                })
            }) {
                if let ApplyOperation::Color { pipeline } = &stage.operation {
                    color(Some(pipeline), &mut output);
                }
            }
        }
    }
    output
}

fn source(
    envelope: &ProjectEnvelope,
    source: &ClipSource,
    demand: ClipDemand,
    output: &mut BTreeMap<MaterialId, MaterialUse>,
) {
    match source {
        ClipSource::Media { material_id } => add(
            output,
            material_id,
            MaterialUse {
                video: demand.visual,
                audio: demand.audio(),
                ..MaterialUse::default()
            },
        ),
        ClipSource::FreezeFrame { material_id, .. } if demand.visual => add(
            output,
            material_id,
            MaterialUse {
                video: true,
                ..MaterialUse::default()
            },
        ),
        ClipSource::Multicam { group_id, switches } => {
            let Some(group) = envelope
                .project
                .multicam_groups
                .iter()
                .find(|group| group.id == *group_id)
            else {
                return;
            };
            for angle in group
                .angles
                .iter()
                .filter(|angle| switches.iter().any(|value| value.angle_id == angle.id))
            {
                add(
                    output,
                    &angle.material_id,
                    MaterialUse {
                        video: demand.visual,
                        audio: demand.audio(),
                        ..MaterialUse::default()
                    },
                );
            }
        }
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. }
            if demand.text == super::TextDemand::Styled =>
        {
            for font in style.font_refs() {
                if let FontRef::Material { material_id } = font {
                    add(
                        output,
                        material_id,
                        MaterialUse {
                            font: true,
                            ..MaterialUse::default()
                        },
                    );
                }
            }
        }
        _ => {}
    }
}

fn color(
    pipeline: Option<&veac_ir::ColorPipeline>,
    output: &mut BTreeMap<MaterialId, MaterialUse>,
) {
    for stage in pipeline.into_iter().flat_map(|value| &value.stages) {
        if let ColorStage::Lut { application } = stage {
            add(
                output,
                &application.material_id,
                MaterialUse {
                    lut: true,
                    ..MaterialUse::default()
                },
            );
        }
    }
}

fn add(output: &mut BTreeMap<MaterialId, MaterialUse>, id: &MaterialId, usage: MaterialUse) {
    output.entry(id.clone()).or_default().merge(usage);
}
