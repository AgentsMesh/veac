use std::collections::BTreeSet;

use crate::ResolvedRenderPlan;

use super::Validator;

impl Validator {
    pub(super) fn graph(&mut self, plan: &ResolvedRenderPlan) {
        if plan.output.sequence_id != plan.entry_sequence_id {
            self.push(
                "PLAN_OUTPUT_SEQUENCE",
                "/output/sequence_id",
                "output sequence must equal the plan entry sequence",
            );
        }
        let mut sequences = BTreeSet::new();
        let mut items = BTreeSet::new();
        let mut entry_found = false;
        for (sequence_index, sequence) in plan.sequences.iter().enumerate() {
            entry_found |= sequence.id == plan.entry_sequence_id;
            if veac_ir::SequenceId::new(sequence.id.as_str()).is_err()
                || !sequences.insert(sequence.id.as_str())
            {
                self.push(
                    "PLAN_SEQUENCE_DUPLICATE",
                    format!("/sequences/{sequence_index}/id"),
                    "sequence IDs must be valid and unique",
                );
            }
            for (track_index, track) in sequence.tracks.iter().enumerate() {
                for (clip_index, clip) in track.clips.iter().enumerate() {
                    if veac_ir::ItemId::new(clip.id.as_str()).is_err()
                        || !items.insert(clip.id.as_str())
                    {
                        self.push(
                            "PLAN_ITEM_DUPLICATE",
                            format!(
                                "/sequences/{sequence_index}/tracks/{track_index}/clips/{clip_index}/id"
                            ),
                            "item IDs must be valid and unique",
                        );
                    }
                }
            }
        }
        if !entry_found {
            self.push(
                "PLAN_ENTRY_SEQUENCE",
                "/entry_sequence_id",
                "entry sequence does not exist in the resolved graph",
            );
        }
        if !strictly_sorted(plan.inputs.iter().map(|value| value.id.as_str())) {
            self.push(
                "PLAN_INPUT_ORDER",
                "/inputs",
                "plan inputs must be uniquely sorted by ID",
            );
        }
    }
}

fn strictly_sorted<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let values = values.collect::<Vec<_>>();
    values.windows(2).all(|pair| pair[0] < pair[1])
}
