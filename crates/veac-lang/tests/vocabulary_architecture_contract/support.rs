use std::{fs, path::PathBuf};

pub(super) const CONTROL_CONSUMER_DIRS: &[&str] = &["src/program/parser", "src/program/index"];

pub(super) fn direct_comparisons(control: &str) -> Vec<String> {
    [
        format!(r#"Some("{control}""#),
        format!(r#"== "{control}""#),
        format!(r#"!= "{control}""#),
        format!(r#""{control}" =>"#),
        format!(r#""{control}" |"#),
        format!(r#"| "{control}""#),
        format!(r#"starts_with("{control}")"#),
        format!(r#"strip_prefix("{control}")"#),
        format!(r#".then("{control}")"#),
        format!(r#".literal("{control}")"#),
    ]
    .into_iter()
    .collect()
}

pub(super) fn contains_word(source: &str, spelling: &str) -> bool {
    if !spelling
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return false;
    }
    source.match_indices(spelling).any(|(start, _)| {
        let before = source[..start].chars().next_back();
        let after = source[start + spelling.len()..].chars().next();
        !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
    })
}

pub(super) fn raw_sink_literals(line: &str, include_format: bool) -> Vec<String> {
    let literals = string_literals(line);
    if line.contains(".push_str(")
        || line.contains("write!(")
        || line.contains("writeln!(")
        || line.contains("\".len()")
    {
        return literals;
    }
    if include_format && line.contains("format!(") {
        return literals
            .into_iter()
            .map(|value| without_format_fields(&value))
            .filter(|value| value.contains(';') || value.contains("\\n") || value.contains(" {{"))
            .collect();
    }
    Vec::new()
}

fn is_word_character(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-')
}

fn string_literals(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut values = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] != b'"' {
            cursor += 1;
            continue;
        }
        let start = cursor + 1;
        cursor = start;
        while cursor < bytes.len() {
            if bytes[cursor] == b'"' && !escaped(bytes, cursor) {
                values.push(source[start..cursor].to_owned());
                cursor += 1;
                break;
            }
            cursor += 1;
        }
    }
    values
}

fn escaped(bytes: &[u8], index: usize) -> bool {
    bytes[..index]
        .iter()
        .rev()
        .take_while(|value| **value == b'\\')
        .count()
        % 2
        == 1
}

fn without_format_fields(value: &str) -> String {
    let mut result = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '{' && characters.peek() != Some(&'{') {
            for value in characters.by_ref() {
                if value == '}' {
                    break;
                }
            }
        } else {
            result.push(character);
            if matches!(character, '{' | '}') && characters.peek() == Some(&character) {
                characters.next();
            }
        }
    }
    result
}

pub(super) fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn production_sources(relative: &str) -> Vec<PathBuf> {
    rust_sources(relative)
        .into_iter()
        .filter(|path| {
            !path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.contains("test"))
        })
        .collect()
}

pub(super) fn read(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub(super) fn read_relative(relative: &str) -> String {
    read(&manifest_dir().join(relative))
}

pub(super) fn rust_sources(relative: &str) -> Vec<PathBuf> {
    let mut pending = vec![manifest_dir().join(relative)];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(
                fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        } else if path.extension().is_some_and(|value| value == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}
