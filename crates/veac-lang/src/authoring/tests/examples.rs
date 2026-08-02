use std::fs;
use std::path::{Path, PathBuf};

use crate::authoring::{format_document, lower_document, parse};
use crate::program::compile_path;

fn example_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut sources = fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path().join("main.veac"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    sources.sort();
    sources
}

#[test]
fn public_pipeline_round_trips_and_lowers_every_example() {
    let sources = example_sources();
    assert!(!sources.is_empty());
    for path in sources {
        let compiled = compile_path(&path)
            .unwrap_or_else(|values| panic!("{} compile failed: {values:?}", path.display()));
        let formatted = format_document(compiled.document());
        let reparsed = parse(&formatted)
            .unwrap_or_else(|values| panic!("{} reparse failed: {values:?}", path.display()));
        assert_eq!(format_document(&reparsed), formatted, "{}", path.display());
        let envelope = lower_document(compiled.document())
            .unwrap_or_else(|values| panic!("{} lowering failed: {values:?}", path.display()));
        veac_ir::validate(&envelope)
            .unwrap_or_else(|values| panic!("{} invalid IR: {values:?}", path.display()));
    }
}

#[test]
fn partial_and_locally_damaged_examples_are_diagnostic() {
    for path in example_sources() {
        let source = fs::read_to_string(&path).unwrap();
        let lines = source.lines().collect::<Vec<_>>();
        for end in 1..lines.len() {
            exercise_candidate(&lines[..end].join("\n"));
        }
        for omitted in 0..lines.len() {
            let candidate = lines
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != omitted)
                .map(|(_, line)| *line)
                .collect::<Vec<_>>()
                .join("\n");
            exercise_candidate(&candidate);
        }
        for (from, to) in [
            (" = true", " = false"),
            (" = false", " = true"),
            (" = 1", " = -1"),
            (" = 0", " = -1"),
            ("linear", "hold"),
            ("item(", "track("),
            ("video(", "audio("),
        ] {
            if source.contains(from) {
                exercise_candidate(&source.replacen(from, to, 1));
            }
        }
        exercise_token_damage(&source);
    }
    for source in [
        super::outputs::OUTPUTS,
        super::modifiers::MODIFIERS,
        super::lowering::SETTINGS,
        super::relations_typed::ALL_RELATIONS,
    ] {
        exercise_token_damage(source);
    }
}

fn exercise_token_damage(source: &str) {
    for (start, end) in token_spans(source) {
        exercise_candidate(&source[..start]);
        let mut removed = source.to_owned();
        removed.replace_range(start..end, "");
        exercise_candidate(&removed);
        let token = &source[start..end];
        let replacement = if token.starts_with('"') {
            "\"\""
        } else if token.parse::<f64>().is_ok() {
            "-1"
        } else if token.starts_with('#') {
            "#GG"
        } else {
            "unknown"
        };
        let mut replaced = source.to_owned();
        replaced.replace_range(start..end, replacement);
        exercise_candidate(&replaced);
        for alternative in super::example_values::alternatives(token) {
            let mut candidate = source.to_owned();
            candidate.replace_range(start..end, &alternative);
            exercise_candidate(&candidate);
        }
    }
}

fn token_spans(source: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    let mut quoted = false;
    let mut escaped = false;
    for (index, value) in source.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == '"' {
                quoted = false;
                spans.push((start.take().unwrap(), index + value.len_utf8()));
            }
            continue;
        }
        if value == '"' {
            if let Some(begin) = start.take() {
                spans.push((begin, index));
            }
            quoted = true;
            start = Some(index);
        } else if value.is_whitespace() || "{}()[]=,".contains(value) {
            if let Some(begin) = start.take() {
                spans.push((begin, index));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(begin) = start {
        spans.push((begin, source.len()));
    }
    spans
}

fn exercise_candidate(source: &str) {
    match parse(source) {
        Ok(document) => {
            let _ = lower_document(&document);
        }
        Err(diagnostics) => assert!(!diagnostics.as_slice().is_empty()),
    }
}
