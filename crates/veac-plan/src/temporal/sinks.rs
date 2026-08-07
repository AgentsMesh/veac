mod apply;
mod clip;
mod values;

use veac_ir::{Animatable, TemporalType};

use crate::ResolvedSequence;

use super::{TemporalSink, TemporalSinkScope};

pub(crate) fn collect(sequences: &[ResolvedSequence]) -> Vec<TemporalSink> {
    let mut collector = Collector { values: Vec::new() };
    for (sequence_index, sequence) in sequences.iter().enumerate() {
        for (track_index, track) in sequence.tracks.iter().enumerate() {
            for (clip_index, clip) in track.clips.iter().enumerate() {
                let pointer =
                    format!("/sequences/{sequence_index}/tracks/{track_index}/clips/{clip_index}");
                let scope = TemporalSinkScope {
                    sequence_id: sequence.id.clone(),
                    item_id: Some(clip.id.clone()),
                };
                collector.clip(clip, &pointer, &scope);
            }
        }
        for (apply_index, apply) in sequence.applies.iter().enumerate() {
            let pointer = format!("/sequences/{sequence_index}/applies/{apply_index}");
            let scope = TemporalSinkScope {
                sequence_id: sequence.id.clone(),
                item_id: None,
            };
            collector.apply(apply, &pointer, &scope);
        }
    }
    collector.values
}

struct Collector {
    values: Vec<TemporalSink>,
}

impl Collector {
    fn leaf<T>(
        &mut self,
        value: &Animatable<T>,
        expected: TemporalType,
        pointer: &str,
        scope: &TemporalSinkScope,
    ) {
        if let Some(binding_id) = value.binding_id() {
            self.values.push(TemporalSink {
                binding_id: binding_id.clone(),
                expected,
                pointer: pointer.to_owned(),
                scope: scope.clone(),
            });
        }
    }
}
