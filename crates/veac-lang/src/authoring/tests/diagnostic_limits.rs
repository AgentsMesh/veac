use crate::authoring::{lower_document, parse, Diagnostics};

use super::project;

const LIMIT: usize = 256;
const LIMIT_CODE: &str = "AUTHORING_DIAGNOSTIC_LIMIT";

#[test]
fn public_parser_bounds_recoverable_syntax_diagnostics() {
    let invalid_members = "  unknown;\n".repeat(LIMIT + 64);
    let errors = parse(&project(&format!("sequence main {{}}\n{invalid_members}"))).unwrap_err();

    assert_bounded(errors);
}

#[test]
fn public_parser_bounds_post_parse_validation_diagnostics() {
    let duplicate_sequences = "  sequence main {}\n".repeat(LIMIT + 64);
    let errors = parse(&project(&duplicate_sequences)).unwrap_err();

    assert_bounded(errors);
}

#[test]
fn public_lowering_bounds_diagnostics() {
    let source = project(
        r#"sequence main {
  layer visual content {
    item sample {
      source generated transparent;
      record { at 0s; duration 1s; }
    }
  }
}"#,
    );
    let mut document = parse(&source).unwrap();
    let template = document.project.sequences[0].clone();
    document.project.sequences = (0..LIMIT + 64)
        .map(|index| {
            let mut sequence = template.clone();
            sequence.id.value = if index == 0 {
                "main".to_owned()
            } else {
                format!("sequence-{index}")
            };
            sequence.layers[0].id.value = format!("content-{index}");
            sequence.layers[0].items[0].id.value = format!("sample-{index}");
            sequence.layers[0].items[0].record.at.raw = "invalid".to_owned();
            sequence
        })
        .collect();

    assert_bounded(lower_document(&document).unwrap_err());
}

fn assert_bounded(errors: Diagnostics) {
    let diagnostics = errors.as_slice();
    assert_eq!(diagnostics.len(), LIMIT);
    assert_eq!(diagnostics.last().unwrap().code, LIMIT_CODE);
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == LIMIT_CODE)
            .count(),
        1
    );
}
