use crate::*;

use super::{SinkScope, TemporalContract};

impl TemporalContract<'_, '_> {
    pub(super) fn leaf<T>(
        &mut self,
        value: &Animatable<T>,
        expected: TemporalType,
        pointer: &str,
        scope: SinkScope<'_>,
    ) {
        let Some(id) = value.binding_id() else {
            return;
        };
        self.used_bindings.insert(id.to_string());
        let Some(index) = self.bindings.get(id.as_str()).copied() else {
            self.push(
                "TEMPORAL_SINK_BINDING_MISSING",
                format!("{pointer}/binding_id"),
                "animation sink references a missing temporal binding",
            );
            return;
        };
        let binding = &self.library.bindings[index];
        if binding.result_type != expected {
            self.push(
                "TEMPORAL_SINK_TYPE",
                pointer,
                "animation sink type does not match its temporal binding",
            );
        }
        if let Some(program) = self
            .library
            .programs
            .iter()
            .find(|program| program.id == binding.program_id)
        {
            if program.result_type != expected {
                self.push(
                    "TEMPORAL_SINK_PROGRAM_TYPE",
                    pointer,
                    "animation sink type does not match its temporal program",
                );
            }
        }
        let owners = binding
            .clocks
            .iter()
            .map(|clock| clock.owner.clone())
            .collect::<Vec<_>>();
        for owner in owners {
            if !scope.matches(&owner) {
                self.push(
                    "TEMPORAL_SINK_CLOCK_OWNER",
                    pointer,
                    "temporal clock owner does not match the animation sink owner",
                );
            }
        }
    }

    pub(super) fn binding_owners(&mut self) {
        let owners = self
            .library
            .bindings
            .iter()
            .flat_map(|binding| &binding.clocks)
            .map(|clock| clock.owner.clone())
            .collect::<Vec<_>>();
        for owner in owners {
            let exists = match &owner {
                TemporalClockOwner::Sequence { sequence_id } => {
                    self.sequence_ids.contains(sequence_id.as_str())
                }
                TemporalClockOwner::Item { item_id } => {
                    self.item_sequences.contains_key(item_id.as_str())
                }
            };
            if !exists {
                self.push(
                    "TEMPORAL_CLOCK_OWNER_MISSING",
                    "/temporal/bindings",
                    "temporal clock owner does not exist in the project graph",
                );
            }
        }
    }
}

impl SinkScope<'_> {
    fn matches(&self, owner: &TemporalClockOwner) -> bool {
        match owner {
            TemporalClockOwner::Sequence { sequence_id } => {
                sequence_id.as_str() == self.sequence_id
            }
            TemporalClockOwner::Item { item_id } => self.item_id == Some(item_id.as_str()),
        }
    }
}
