use std::collections::{BTreeMap, BTreeSet};

use crate::temporal::{
    temporal_program_digest, TemporalInputSource, TemporalProgram, TemporalProgramLibrary,
    MAX_TEMPORAL_INPUTS, MAX_TEMPORAL_NODES_PER_PROGRAM, TEMPORAL_OPSET_VERSION,
};

use super::Validator;

impl Validator {
    pub(super) fn library(&mut self, library: &TemporalProgramLibrary) {
        if library.opset_version != TEMPORAL_OPSET_VERSION {
            self.push(
                "TEMPORAL_OPSET",
                "/opset_version",
                "unsupported temporal opset version",
            );
        }
        self.library_limits(library);
        let mut provenance = BTreeSet::new();
        for (index, value) in library.provenance.iter().enumerate() {
            self.provenance(value, &format!("/provenance/{index}"));
            if !provenance.insert(value.id.as_str().to_owned()) {
                self.push(
                    "TEMPORAL_PROVENANCE_DUPLICATE",
                    format!("/provenance/{index}/id"),
                    "duplicate provenance ID",
                );
            }
        }
        let mut programs = BTreeMap::new();
        let mut digests = BTreeSet::new();
        for (index, program) in library.programs.iter().enumerate() {
            let pointer = format!("/programs/{index}");
            self.program(program, &pointer);
            if programs
                .insert(program.id.as_str().to_owned(), program)
                .is_some()
            {
                self.push(
                    "TEMPORAL_PROGRAM_DUPLICATE",
                    format!("{pointer}/id"),
                    "duplicate program ID",
                );
            }
            if !digests.insert(program.content_sha256.as_str()) {
                self.push(
                    "TEMPORAL_PROGRAM_CONTENT_DUPLICATE",
                    format!("{pointer}/content_sha256"),
                    "duplicate program content must share one definition",
                );
            }
            self.program_provenance(program, &provenance, &pointer);
        }
        let mut bindings = BTreeSet::new();
        for (index, binding) in library.bindings.iter().enumerate() {
            let pointer = format!("/bindings/{index}");
            self.binding(binding, &programs, &provenance, &pointer);
            if !bindings.insert(binding.id.as_str()) {
                self.push(
                    "TEMPORAL_BINDING_DUPLICATE",
                    format!("{pointer}/id"),
                    "duplicate binding ID",
                );
            }
        }
    }

    pub(super) fn program(&mut self, program: &TemporalProgram, pointer: &str) {
        if !program.id.is_valid() {
            self.push(
                "TEMPORAL_PROGRAM_ID",
                format!("{pointer}/id"),
                "invalid temporal program ID",
            );
        }
        if program.opset_version != TEMPORAL_OPSET_VERSION {
            self.push(
                "TEMPORAL_OPSET",
                format!("{pointer}/opset_version"),
                "unsupported temporal opset version",
            );
        }
        if !program.provenance_id.is_valid() {
            self.push(
                "TEMPORAL_PROVENANCE_ID",
                format!("{pointer}/provenance_id"),
                "invalid provenance ID",
            );
        }
        self.inputs(program, pointer);
        self.nodes(program, pointer);
        match temporal_program_digest(program) {
            Ok(digest) if digest == program.content_sha256 => {}
            Ok(_) => self.push(
                "TEMPORAL_PROGRAM_DIGEST",
                format!("{pointer}/content_sha256"),
                "program content digest does not match",
            ),
            Err(error) => self.push(
                "TEMPORAL_PROGRAM_DIGEST",
                format!("{pointer}/content_sha256"),
                error.to_string(),
            ),
        }
    }

    fn inputs(&mut self, program: &TemporalProgram, pointer: &str) {
        if program.inputs.len() > MAX_TEMPORAL_INPUTS {
            self.push(
                "TEMPORAL_INPUT_LIMIT",
                format!("{pointer}/inputs"),
                "program input limit exceeded",
            );
        }
        let mut sources = BTreeSet::new();
        for (index, input) in program.inputs.iter().enumerate() {
            let path = format!("{pointer}/inputs/{index}");
            if input.id.get() != u32::try_from(index).unwrap_or(u32::MAX) {
                self.push(
                    "TEMPORAL_INPUT_ORDER",
                    format!("{path}/id"),
                    "input IDs must be contiguous and ordered",
                );
            }
            match &input.source {
                TemporalInputSource::Clock { clock } if input.value_type != clock.value_type() => {
                    self.push(
                        "TEMPORAL_INPUT_TYPE",
                        format!("{path}/value_type"),
                        "clock input type does not match its clock",
                    );
                }
                TemporalInputSource::Parameter { parameter_id } if !parameter_id.is_valid() => {
                    self.push(
                        "TEMPORAL_PARAMETER_ID",
                        format!("{path}/source/parameter_id"),
                        "invalid parameter ID",
                    );
                }
                _ => {}
            }
            if !sources.insert(input.source.clone()) {
                self.push(
                    "TEMPORAL_INPUT_SOURCE_DUPLICATE",
                    format!("{path}/source"),
                    "input source is duplicated",
                );
            }
        }
    }
}

pub(super) fn per_program_limit() -> usize {
    MAX_TEMPORAL_NODES_PER_PROGRAM
}
