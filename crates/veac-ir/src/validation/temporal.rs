mod binding;
mod reachability;
mod sinks;

use std::collections::{BTreeMap, BTreeSet};

use crate::*;

use super::Validator;

pub(super) struct TemporalContract<'a, 'b> {
    canonical: &'a mut Validator,
    library: &'b TemporalProgramLibrary,
    bindings: BTreeMap<&'b str, usize>,
    used_bindings: BTreeSet<String>,
    sequence_ids: BTreeSet<&'b str>,
    item_sequences: BTreeMap<&'b str, &'b str>,
}

#[derive(Clone, Copy)]
pub(super) struct SinkScope<'a> {
    sequence_id: &'a str,
    item_id: Option<&'a str>,
}

impl<'a> SinkScope<'a> {
    fn sequence(sequence_id: &'a str) -> Self {
        Self {
            sequence_id,
            item_id: None,
        }
    }

    fn item(sequence_id: &'a str, item_id: &'a str) -> Self {
        Self {
            sequence_id,
            item_id: Some(item_id),
        }
    }
}

impl Validator {
    pub(super) fn temporal_contract(
        &mut self,
        library: &TemporalProgramLibrary,
        project: &Project,
    ) {
        if let Err(errors) = validate_temporal_library(library) {
            for diagnostic in errors.into_diagnostics() {
                self.push(
                    diagnostic.code,
                    None,
                    format!("/temporal{}", diagnostic.pointer),
                    diagnostic.message,
                    None,
                );
            }
        }
        TemporalContract::new(self, library, project).validate(project);
    }
}

impl<'a, 'b> TemporalContract<'a, 'b> {
    fn new(
        canonical: &'a mut Validator,
        library: &'b TemporalProgramLibrary,
        project: &'b Project,
    ) -> Self {
        let bindings = library
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| (binding.id.as_str(), index))
            .collect();
        let sequence_ids = project
            .sequences
            .iter()
            .map(|sequence| sequence.id.as_str())
            .collect();
        let item_sequences = project
            .sequences
            .iter()
            .flat_map(|sequence| {
                sequence
                    .tracks
                    .iter()
                    .flat_map(|track| &track.clips)
                    .map(|clip| (clip.id.as_str(), sequence.id.as_str()))
            })
            .collect();
        Self {
            canonical,
            library,
            bindings,
            used_bindings: BTreeSet::new(),
            sequence_ids,
            item_sequences,
        }
    }

    fn validate(mut self, project: &Project) {
        self.binding_owners();
        self.project_sinks(project);
        self.reachability();
    }

    fn push(&mut self, code: &str, pointer: impl Into<String>, message: impl Into<String>) {
        self.canonical.push(code, None, pointer, message, None);
    }
}
