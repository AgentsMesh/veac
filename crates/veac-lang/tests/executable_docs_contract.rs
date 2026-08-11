use std::{collections::BTreeSet, fs, path::PathBuf};

use veac_lang::program::{build_source, DomainOperationId, DomainType};
use veac_lang::vocabulary::{language_spec, GrammarPosition};

const REFERENCE: &str = "docs/language-reference/executable-build.md";

#[test]
fn executable_reference_program_builds_to_canonical_ir() {
    let source = read(REFERENCE);
    let blocks = fenced_blocks(&source, "veac,executable");
    assert_eq!(blocks.len(), 1, "{REFERENCE} must contain one full program");
    let built = build_source(blocks[0])
        .unwrap_or_else(|errors| panic!("{REFERENCE} does not build: {errors}"));
    let envelope = built.envelope();
    assert_eq!(envelope.project.sequences.len(), 1);
    assert_eq!(envelope.project.sequences[0].tracks[0].clips.len(), 2);
    veac_ir::validate(envelope)
        .unwrap_or_else(|errors| panic!("{REFERENCE} emits invalid IR: {errors:?}"));
}

#[test]
fn executable_rfc_surface_examples_use_the_real_language() {
    for relative in [
        "docs/rfcs/executable-veac-language.md",
        "docs/rfcs/executable-domain-graph-values.md",
    ] {
        let source = read(relative);
        let blocks = fenced_blocks(&source, "veac");
        assert_eq!(blocks.len(), 1, "{relative} must contain one full program");
        build_source(blocks[0])
            .unwrap_or_else(|errors| panic!("{relative} does not build: {errors}"));
    }
}

#[test]
fn executable_docs_track_the_closed_runtime_boundary() {
    let source = read(REFERENCE);
    assert_eq!(DomainType::all().len(), 214);
    assert_eq!(DomainOperationId::all().len(), 582);
    for contract in [
        "verified Core v10",
        "Pure | LocalMutation | GraphEmit",
        "Const < Build < Temporal",
        "static topology, dynamic leaf values",
        "filter` 与 `fold`",
        "direct canonical ProjectEnvelope",
        "只走 executable pipeline",
        "不是 lexer keyword",
    ] {
        assert!(
            source.contains(contract),
            "missing documented contract: {contract}"
        );
    }
    assert!(read("docs/language-reference/README.md").contains(
        "[可执行 Build、Core v10、Effect/Stage 与 graph transaction](executable-build.md)"
    ));
    assert!(read("README.md")
        .contains("[Executable Build](docs/language-reference/executable-build.md)"));
}

#[test]
fn normative_programming_grammar_tracks_effect_input_and_statement_contracts() {
    let grammar = read("docs/language-design/programming-grammar.md");
    for contract in [
        "input-role = \"parameter\" | \"asset_metadata\" | \"analysis\" | \"material\"",
        "block-statement = let-statement | var-statement | set-statement",
        "function-effect = \"effect\" , ( \"pure\" | \"local\" | \"emit\" | \"any\" )",
        "value-type , function-effect , function-body",
    ] {
        assert!(
            grammar.contains(contract),
            "normative grammar is missing: {contract}"
        );
    }
}

#[test]
fn normative_grammar_tracks_the_exact_temporal_vocabulary() {
    let grammar = read("docs/language-design/programming-grammar.md");
    for (production, position) in [
        ("temporal-target", GrammarPosition::TemporalTargetKind),
        (
            "temporal-property",
            GrammarPosition::TemporalPropertyPosition,
        ),
    ] {
        assert_eq!(
            production_literals(&grammar, production),
            vocabulary_at(position),
            "{production} drifted from the production vocabulary"
        );
    }
    assert!(grammar.contains("executable-temporal-sinks.md"));
}

fn vocabulary_at(position: GrammarPosition) -> BTreeSet<String> {
    language_spec()
        .vocabulary
        .entries
        .into_iter()
        .filter(|entry| entry.uses.iter().any(|usage| usage.position == position))
        .map(|entry| entry.spelling)
        .collect()
}

fn production_literals(source: &str, production: &str) -> BTreeSet<String> {
    let prefix = format!("{production} =");
    let mut body = String::new();
    for line in source
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(&prefix))
    {
        body.push_str(line);
        if line.contains(';') {
            break;
        }
    }
    assert!(body.contains(';'), "missing EBNF production {production}");
    body.split('"')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, value)| value.to_owned())
        .collect()
}

fn fenced_blocks<'a>(source: &'a str, language: &str) -> Vec<&'a str> {
    let marker = format!("```{language}\n");
    source
        .split(&marker)
        .skip(1)
        .map(|tail| tail.split_once("\n```").expect("closed code fence").0)
        .collect()
}

fn read(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
