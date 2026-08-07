use crate::*;

use super::{opcode_is_current, Validator, MAX_CALL_STACK, MAX_ITERATIONS};

impl Validator {
    pub(super) fn authorship_event(
        &mut self,
        value: &AuthorshipEvent,
        path: &str,
        object_id: &str,
    ) {
        if !opcode_is_current(value.operation) {
            self.value_error("AUTHORSHIP_OPCODE", path, object_id);
        }
        if value.call_stack.len() > MAX_CALL_STACK || value.iterations.len() > MAX_ITERATIONS {
            self.value_error("AUTHORSHIP_DEPTH", path, object_id);
        }
        self.definition(&value.definition, &format!("{path}/definition"), object_id);
        self.site(&value.origin, &format!("{path}/origin"), object_id);
        if value.origin.definition != value.definition.identity {
            self.value_error("AUTHORSHIP_DEFINITION", path, object_id);
        }
        for (index, site) in value.call_stack.iter().enumerate() {
            self.site(site, &format!("{path}/call_stack/{index}"), object_id);
        }
        for (index, iteration) in value.iterations.iter().enumerate() {
            self.iteration(iteration, &format!("{path}/iterations/{index}"), object_id);
        }
    }

    fn definition(&mut self, value: &AuthoredDefinition, path: &str, object_id: &str) {
        if !value.identity.is_valid(256)
            || !value.name.is_valid(256)
            || !value.source.is_valid(1_024)
            || !span_valid(value.span)
        {
            self.value_error("AUTHORSHIP_DEFINITION", path, object_id);
        }
    }

    fn site(&mut self, value: &AuthoredSite, path: &str, object_id: &str) {
        if !value.definition.is_valid(256)
            || !value.function.is_valid(256)
            || !value.source.is_valid(1_024)
            || !span_valid(value.span)
        {
            self.value_error("AUTHORSHIP_SITE", path, object_id);
        }
    }

    fn iteration(&mut self, value: &AuthoredIteration, path: &str, object_id: &str) {
        if !value.definition.is_valid(256)
            || !value.function.is_valid(256)
            || !value.source.is_valid(1_024)
            || !value.logical_key.is_valid(69)
            || !valid_loop_key(value.logical_key.as_str())
            || !span_valid(value.loop_span)
            || !span_valid(value.binding_span)
            || value.index > MAX_SAFE_INTEGER
        {
            self.value_error("AUTHORSHIP_ITERATION", path, object_id);
        }
    }
}

fn span_valid(value: AuthoredSpan) -> bool {
    value.start <= value.end && value.end <= MAX_SAFE_INTEGER
}

fn valid_loop_key(value: &str) -> bool {
    value.strip_prefix("loop_").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
