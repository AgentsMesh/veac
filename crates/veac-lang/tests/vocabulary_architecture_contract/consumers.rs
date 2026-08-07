use veac_lang::vocabulary::language_spec;

use super::support::{
    contains_word, direct_comparisons, production_sources, raw_sink_literals, read,
    CONTROL_CONSUMER_DIRS,
};

#[test]
fn production_consumers_cannot_bypass_control_uses() {
    for directory in CONTROL_CONSUMER_DIRS {
        for path in production_sources(directory) {
            let source = read(&path);
            assert!(!source.contains("ControlWord::"), "{}", path.display());
            assert!(
                !source.contains("vocabulary::ControlWord"),
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn registered_syntax_is_never_consumed_as_a_raw_spelling() {
    let vocabulary = language_spec().vocabulary;
    for directory in CONTROL_CONSUMER_DIRS {
        for path in production_sources(directory) {
            let source = read(&path);
            for entry in &vocabulary.entries {
                for needle in direct_comparisons(&entry.spelling) {
                    assert!(
                        !source.contains(&needle),
                        "{} compares {} through {needle:?}",
                        path.display(),
                        entry.spelling
                    );
                }
            }
        }
    }
}

#[test]
fn registered_syntax_cannot_enter_raw_source_writers() {
    let spellings = language_spec()
        .vocabulary
        .entries
        .into_iter()
        .map(|entry| entry.spelling)
        .collect::<Vec<_>>();
    let mut violations = Vec::new();
    for directory in CONTROL_CONSUMER_DIRS {
        for path in production_sources(directory) {
            let generated_source = path.to_string_lossy().contains("/program/index/");
            for (index, line) in read(&path).lines().enumerate() {
                for literal in raw_sink_literals(line, generated_source) {
                    for spelling in &spellings {
                        if contains_word(&literal, spelling) {
                            violations.push(format!(
                                "{}:{} writes registered {spelling:?} through a raw string",
                                path.display(),
                                index + 1
                            ));
                        }
                    }
                }
            }
        }
    }
    violations.sort();
    violations.dedup();
    assert!(
        violations.is_empty(),
        "raw registered syntax emission:\n{}",
        violations.join("\n")
    );
}
