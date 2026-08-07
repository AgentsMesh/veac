use std::collections::{BTreeMap, BTreeSet};

use crate::temporal::{
    TemporalBinding, TemporalClock, TemporalClockOwner, TemporalInputDeclaration,
    TemporalInputSource, TemporalProgram,
};

use super::Validator;

impl Validator {
    pub(super) fn binding(
        &mut self,
        binding: &TemporalBinding,
        programs: &BTreeMap<String, &TemporalProgram>,
        provenance: &BTreeSet<String>,
        pointer: &str,
    ) {
        if !binding.id.is_valid() {
            self.push(
                "TEMPORAL_BINDING_ID",
                format!("{pointer}/id"),
                "invalid binding ID",
            );
        }
        if !binding.provenance_id.is_valid() || !provenance.contains(binding.provenance_id.as_str())
        {
            self.push(
                "TEMPORAL_PROVENANCE_MISSING",
                format!("{pointer}/provenance_id"),
                "binding provenance does not exist",
            );
        }
        let Some(program) = programs.get(binding.program_id.as_str()) else {
            self.push(
                "TEMPORAL_PROGRAM_MISSING",
                format!("{pointer}/program_id"),
                "binding program does not exist",
            );
            return;
        };
        if binding.result_type != program.result_type {
            self.push(
                "TEMPORAL_BINDING_RESULT",
                format!("{pointer}/result_type"),
                "binding result type does not match its program",
            );
        }
        let declarations: BTreeMap<_, _> = program
            .inputs
            .iter()
            .map(|input| (input.id.get(), input))
            .collect();
        let mut bound = BTreeSet::new();
        for (index, clock) in binding.clocks.iter().enumerate() {
            let path = format!("{pointer}/clocks/{index}");
            self.bind_once(clock.input_id.get(), &mut bound, &path);
            self.clock(
                clock,
                declarations.get(&clock.input_id.get()).copied(),
                &path,
            );
        }
        for (index, parameter) in binding.parameters.iter().enumerate() {
            let path = format!("{pointer}/parameters/{index}");
            self.bind_once(parameter.input_id.get(), &mut bound, &path);
            self.value(&parameter.value, &format!("{path}/value"));
            match declarations
                .get(&parameter.input_id.get())
                .map(|value| &value.source)
            {
                Some(TemporalInputSource::Parameter { parameter_id })
                    if parameter_id == &parameter.parameter_id
                        && parameter.value.value_type()
                            == declarations[&parameter.input_id.get()].value_type => {}
                _ => self.push(
                    "TEMPORAL_PARAMETER_BINDING",
                    path,
                    "parameter binding does not match its declaration",
                ),
            }
        }
        if bound.len() != declarations.len() || declarations.keys().any(|id| !bound.contains(id)) {
            self.push(
                "TEMPORAL_BINDING_INCOMPLETE",
                pointer,
                "binding must bind every program input exactly once",
            );
        }
    }

    fn bind_once(&mut self, input: u32, bound: &mut BTreeSet<u32>, pointer: &str) {
        if !bound.insert(input) {
            self.push(
                "TEMPORAL_INPUT_BINDING_DUPLICATE",
                pointer,
                "program input is bound more than once",
            );
        }
    }

    fn clock(
        &mut self,
        binding: &crate::TemporalClockBinding,
        declaration: Option<&TemporalInputDeclaration>,
        pointer: &str,
    ) {
        let matches = matches!(
            declaration,
            Some(TemporalInputDeclaration { source: TemporalInputSource::Clock { clock }, value_type, .. })
                if *clock == binding.clock && *value_type == binding.clock.value_type()
        );
        let owner = matches!(
            (binding.clock, &binding.owner),
            (TemporalClock::SequenceTime | TemporalClock::Frame, TemporalClockOwner::Sequence { sequence_id }) if sequence_id.is_valid()
        ) || matches!(
            (binding.clock, &binding.owner),
            (TemporalClock::ClipTime | TemporalClock::SourceTime | TemporalClock::Progress, TemporalClockOwner::Item { item_id }) if item_id.is_valid()
        );
        if !matches || !owner {
            self.push(
                "TEMPORAL_CLOCK_BINDING",
                pointer,
                "clock binding does not match its declaration or owner",
            );
        }
    }
}
