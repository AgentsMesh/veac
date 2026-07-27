use std::collections::BTreeSet;

use super::BackendFilterBinding;

const PREFIX: &str = "__VEAC_FILTER_RESOURCE_";
const SUFFIX_BYTES: usize = 6;

pub(super) fn validate(template: &str, bindings: &[BackendFilterBinding]) -> Result<(), String> {
    let mut declared = BTreeSet::new();
    for binding in bindings {
        let token = binding.token();
        if !valid(token) || !declared.insert(token) || template.matches(token).count() != 1 {
            return Err("filter resource tokens must be unique and occur exactly once".into());
        }
        if binding.files().is_empty() {
            return Err("filter resource binding must reference at least one file".into());
        }
    }
    for token in template_tokens(template)? {
        if !declared.contains(token) {
            return Err("filter resource template contains an unbound token".to_owned());
        }
    }
    Ok(())
}

fn valid(value: &str) -> bool {
    let Some(suffix) = value.as_bytes().strip_prefix(PREFIX.as_bytes()) else {
        return false;
    };
    valid_suffix(suffix)
}

fn template_tokens(value: &str) -> Result<Vec<&str>, String> {
    value
        .match_indices(PREFIX)
        .map(|(start, _)| {
            let end = start + PREFIX.len() + SUFFIX_BYTES;
            let token = value
                .get(start..end)
                .ok_or_else(|| "filter resource template contains a malformed token".to_owned())?;
            if valid(token) {
                Ok(token)
            } else {
                Err("filter resource template contains a malformed token".to_owned())
            }
        })
        .collect()
}

fn valid_suffix(value: &[u8]) -> bool {
    value.len() == SUFFIX_BYTES && value[..4].iter().all(u8::is_ascii_digit) && value[4..] == *b"__"
}
