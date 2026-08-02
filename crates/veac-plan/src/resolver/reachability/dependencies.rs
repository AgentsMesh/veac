use veac_ir::{RelationGraph, RelationSignal, Sequence};

use super::{apply, demand::SequenceDemand, ClipDemand};

pub(super) fn close(
    sequence: &Sequence,
    relations: &RelationGraph<'_>,
    clips: impl Fn(&veac_ir::ItemId) -> Option<ClipDemand> + Copy,
    demand: &mut SequenceDemand,
) -> bool {
    let before = demand.clone();
    for edge in relations.mattes(&sequence.id) {
        if clips(&edge.consumer.clip.id).is_some_and(|value| value.visual) {
            demand.visual_controls.insert(edge.producer.clip.id.clone());
        }
    }
    for edge in relations.apply_mattes(&sequence.id) {
        if apply::active(sequence, edge.consumer.apply, clips) {
            demand.visual_controls.insert(edge.producer.clip.id.clone());
        }
    }
    for edge in relations.sidechains(&sequence.id) {
        if !clips(&edge.target.clip.id).is_some_and(|value| value.audio_output) {
            continue;
        }
        let Some(active) = super::super::apply_ranges::absolute(
            edge.target.clip.record_range,
            edge.parameters.active_range,
        ) else {
            continue;
        };
        match edge.key {
            RelationSignal::Track(track) => {
                demand.add_audio_control(&track.id, active);
            }
            RelationSignal::Bus { tracks, .. } => {
                for track in tracks {
                    demand.add_audio_control(&track.id, active);
                }
            }
        }
    }
    *demand != before
}
