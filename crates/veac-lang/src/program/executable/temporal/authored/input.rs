use std::collections::BTreeMap;

use crate::program::expression::CoreTemporalInputIdentity;
use veac_ir::{ItemId, MaterialId, SequenceId};

pub(super) fn clip_clocks(
    item_id: &ItemId,
    sequence_id: &SequenceId,
    source_id: Option<MaterialId>,
) -> BTreeMap<String, CoreTemporalInputIdentity> {
    let mut inputs = sequence_clocks(sequence_id);
    inputs.extend([
        (
            "clip_time".to_owned(),
            CoreTemporalInputIdentity::ClipTime {
                item_id: item_id.clone(),
            },
        ),
        (
            "progress".to_owned(),
            CoreTemporalInputIdentity::Progress {
                item_id: item_id.clone(),
            },
        ),
    ]);
    if let Some(source_id) = source_id {
        inputs.insert(
            "source_time".to_owned(),
            CoreTemporalInputIdentity::SourceTime { source_id },
        );
    }
    inputs
}

pub(super) fn sequence_clocks(
    sequence_id: &SequenceId,
) -> BTreeMap<String, CoreTemporalInputIdentity> {
    BTreeMap::from([
        (
            "sequence_time".to_owned(),
            CoreTemporalInputIdentity::SequenceTime {
                sequence_id: sequence_id.clone(),
            },
        ),
        (
            "frame".to_owned(),
            CoreTemporalInputIdentity::Frame {
                sequence_id: sequence_id.clone(),
            },
        ),
    ])
}
