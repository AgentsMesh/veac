use std::collections::BTreeSet;

use crate::temporal::{
    TemporalAuthoredSite, TemporalDefinitionSite, TemporalProvenance, TemporalSourceSpan,
    MAX_TEMPORAL_CALL_DEPTH, MAX_TEMPORAL_LOGICAL_KEYS,
};

use super::Validator;

impl Validator {
    pub(super) fn provenance(&mut self, value: &TemporalProvenance, pointer: &str) {
        if !value.id.is_valid() {
            self.push(
                "TEMPORAL_PROVENANCE_ID",
                format!("{pointer}/id"),
                "invalid provenance ID",
            );
        }
        self.definition(&value.definition, &format!("{pointer}/definition"));
        self.site(&value.origin, &format!("{pointer}/origin"));
        if value.origin.definition_id != value.definition.id {
            self.push(
                "TEMPORAL_PROVENANCE_ORIGIN",
                format!("{pointer}/origin/definition_id"),
                "origin must name the retained definition",
            );
        }
        if value.call_stack.len() > MAX_TEMPORAL_CALL_DEPTH {
            self.push(
                "TEMPORAL_CALL_DEPTH",
                format!("{pointer}/call_stack"),
                "provenance call stack limit exceeded",
            );
        }
        for (index, site) in value.call_stack.iter().enumerate() {
            self.site(site, &format!("{pointer}/call_stack/{index}"));
        }
        if value.logical_keys.len() > MAX_TEMPORAL_LOGICAL_KEYS {
            self.push(
                "TEMPORAL_LOGICAL_KEY_LIMIT",
                format!("{pointer}/logical_keys"),
                "logical key limit exceeded",
            );
        }
        let mut keys = BTreeSet::new();
        for (index, key) in value.logical_keys.iter().enumerate() {
            if !key.is_valid() {
                self.push(
                    "TEMPORAL_LOGICAL_KEY",
                    format!("{pointer}/logical_keys/{index}"),
                    "invalid logical key",
                );
            }
            if !keys.insert(key.as_str()) {
                self.push(
                    "TEMPORAL_LOGICAL_KEY_DUPLICATE",
                    format!("{pointer}/logical_keys/{index}"),
                    "duplicate logical key",
                );
            }
        }
    }

    fn definition(&mut self, value: &TemporalDefinitionSite, pointer: &str) {
        if !value.id.is_valid() || !value.source_id.is_valid() {
            self.push(
                "TEMPORAL_DEFINITION_ID",
                pointer,
                "definition or source ID is invalid",
            );
        }
        if !name_valid(&value.name) {
            self.push(
                "TEMPORAL_DEFINITION_NAME",
                format!("{pointer}/name"),
                "definition name is invalid",
            );
        }
        self.span(value.span, &format!("{pointer}/span"));
    }

    fn site(&mut self, value: &TemporalAuthoredSite, pointer: &str) {
        if !value.definition_id.is_valid() || !value.source_id.is_valid() {
            self.push("TEMPORAL_SITE_ID", pointer, "authored site ID is invalid");
        }
        if !name_valid(&value.function) {
            self.push(
                "TEMPORAL_SITE_NAME",
                format!("{pointer}/function"),
                "authored function name is invalid",
            );
        }
        self.span(value.span, &format!("{pointer}/span"));
    }

    fn span(&mut self, value: TemporalSourceSpan, pointer: &str) {
        if value.start > value.end {
            self.push(
                "TEMPORAL_SOURCE_SPAN",
                pointer,
                "source span end precedes its start",
            );
        }
    }
}

fn name_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}
