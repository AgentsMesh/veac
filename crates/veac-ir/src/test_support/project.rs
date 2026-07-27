use std::collections::BTreeMap;

use crate::*;

use super::{
    clips::caption_clip,
    materials::{font_material, video_material},
    media_clip,
};

pub(crate) fn sample_project() -> ProjectEnvelope {
    let video = video_material();
    let font = font_material();
    let sequence_id = SequenceId::new("seq_main").unwrap();
    ProjectEnvelope::new(Project {
        id: ProjectId::new("prj_sample").unwrap(),
        revision: 7,
        timebase: 600,
        entry_sequence_id: sequence_id.clone(),
        render_configs: vec![render_config(sequence_id.clone())],
        materials: vec![video, font.clone()],
        multicam_groups: vec![],
        annotations: vec![],
        relations: vec![],
        sequences: vec![Sequence {
            id: sequence_id,
            name: "Main".to_owned(),
            settings: SequenceSettings {
                width: 1080,
                height: 1920,
                frame_rate: Rational::new(30, 1).unwrap(),
                sample_rate: 48_000,
            },
            tracks: vec![
                track(
                    "trk_video",
                    TrackKind::Video,
                    0,
                    PlacementMode::Magnetic,
                    vec![media_clip()],
                ),
                track(
                    "trk_captions",
                    TrackKind::Caption,
                    10,
                    PlacementMode::Free,
                    vec![caption_clip(font.id)],
                ),
            ],
            applies: Vec::new(),
            metadata: BTreeMap::new(),
        }],
        applied_operations: vec![],
        metadata: BTreeMap::new(),
    })
}

pub(crate) fn linked_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[0].placement_mode = PlacementMode::Free;
    let mut audio_clip = sequence.tracks[0].clips[0].clone();
    audio_clip.id = ItemId::new("itm_audio").unwrap();
    audio_clip.visual = None;
    audio_clip.effects.clear();
    sequence.tracks.insert(
        1,
        track(
            "trk_audio",
            TrackKind::Audio,
            5,
            PlacementMode::Free,
            vec![audio_clip],
        ),
    );
    project.project.relations.push(Relation {
        id: RelationId::new("rel_primary").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::AvLink {
            video: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            audio: vec![RelationEndpoint::item(ItemId::new("itm_audio").unwrap())],
        },
    });
    project
}

fn render_config(sequence_id: SequenceId) -> RenderConfig {
    RenderConfig {
        id: RenderConfigId::new("out_main").unwrap(),
        sequence_id,
        width: 1080,
        height: 1920,
        frame_rate: Rational::new(30, 1).unwrap(),
        deliverables: vec![Deliverable {
            id: DeliverableId::new("dlv_main").unwrap(),
            file_name: "main.mp4".to_owned(),
            kind: DeliverableKind::Video(VideoDeliverable::default()),
        }],
    }
}

fn track(
    id: &str,
    kind: TrackKind,
    order: i32,
    placement_mode: PlacementMode,
    clips: Vec<Clip>,
) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind,
        order,
        placement_mode,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    }
}
