use std::collections::BTreeSet;

pub(super) fn names(graph: &str) -> BTreeSet<String> {
    let bytes = graph.as_bytes();
    let mut names = BTreeSet::new();
    let mut index = 0;
    let mut starts_filter = true;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        if starts_filter {
            index = skip_labels_and_space(bytes, index);
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            if index > start {
                names.insert(graph[start..index].to_owned());
            }
            starts_filter = false;
            continue;
        }
        let byte = bytes[index];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'\'' {
            quoted = !quoted;
        } else if !quoted && matches!(byte, b',' | b';') {
            starts_filter = true;
        }
        index += 1;
    }
    names
}

fn skip_labels_and_space(bytes: &[u8], mut index: usize) -> usize {
    loop {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index) != Some(&b'[') {
            return index;
        }
        while index < bytes.len() && bytes[index] != b']' {
            index += 1;
        }
        index = index.saturating_add(1);
    }
}

#[cfg(test)]
#[path = "filter/tests.rs"]
mod tests;
