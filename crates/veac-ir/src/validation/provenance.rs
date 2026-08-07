mod event;
mod generated;
mod owner;

use crate::*;

use super::Validator;

const MAX_PATH_SEGMENTS: usize = 64;
const MAX_EVENTS_PER_ENTITY: usize = 4_096;
const MAX_CALL_STACK: usize = 64;
const MAX_ITERATIONS: usize = 64;
const MAX_AUTHORSHIP_BYTES: usize = 64 * 1024 * 1024;

impl Validator {
    pub(super) fn entity_authorship(
        &mut self,
        value: &EntityAuthorship,
        path: &str,
        object_id: &str,
    ) {
        let logical = value
            .logical_path
            .iter()
            .map(|segment| segment.as_str().to_owned())
            .collect::<Vec<_>>();
        if logical.is_empty()
            || logical.len() > MAX_PATH_SEGMENTS
            || value
                .logical_path
                .iter()
                .any(|segment| !segment.is_valid_logical_key())
            || !self.authorship_paths.insert(logical)
        {
            self.value_error("AUTHORSHIP_PATH", path, object_id);
        }
        if value.events.is_empty() || value.events.len() > MAX_EVENTS_PER_ENTITY {
            self.value_error("AUTHORSHIP_EVENTS", path, object_id);
        }
        self.authorship_bytes = self.authorship_bytes.saturating_add(logical_bytes(value));
        if self.authorship_bytes > MAX_AUTHORSHIP_BYTES {
            self.value_error("AUTHORSHIP_BUDGET", path, object_id);
        }
        for (index, event) in value.events.iter().enumerate() {
            self.authorship_event(event, &format!("{path}/events/{index}"), object_id);
        }
    }
}

fn logical_bytes(value: &EntityAuthorship) -> usize {
    let mut bytes = value
        .logical_path
        .iter()
        .map(|part| part.as_str().len())
        .fold(0usize, usize::saturating_add);
    for event in &value.events {
        bytes = bytes.saturating_add(definition_bytes(&event.definition));
        bytes = bytes.saturating_add(site_bytes(&event.origin));
        bytes = bytes.saturating_add(
            event
                .call_stack
                .iter()
                .map(site_bytes)
                .fold(0usize, usize::saturating_add),
        );
        bytes = bytes.saturating_add(
            event
                .iterations
                .iter()
                .map(iteration_bytes)
                .fold(0usize, usize::saturating_add),
        );
        bytes = bytes.saturating_add(8);
    }
    bytes
}

fn definition_bytes(value: &AuthoredDefinition) -> usize {
    value.identity.as_str().len() + value.name.as_str().len() + value.source.as_str().len() + 24
}

fn site_bytes(value: &AuthoredSite) -> usize {
    value.definition.as_str().len()
        + value.function.as_str().len()
        + value.source.as_str().len()
        + 16
}

fn iteration_bytes(value: &AuthoredIteration) -> usize {
    value.definition.as_str().len()
        + value.function.as_str().len()
        + value.source.as_str().len()
        + value.logical_key.as_str().len()
        + 40
}

fn opcode_is_current(value: DomainOpcode) -> bool {
    generated::is_current(value.0)
}

#[cfg(test)]
#[path = "provenance/tests.rs"]
mod tests;
