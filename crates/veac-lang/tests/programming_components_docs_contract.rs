use std::{fs, path::PathBuf};

use veac_lang::program::{
    build_path_with_inputs, build_source, parse_build_input_manifest, DomainOperationId as Op,
    DomainOperationRegistry,
};

const DOC: &str = "docs/language-reference/programming-components.md";
const AGENT_DOC: &str = "docs/language-design/agent-reuse.md";
const EXAMPLE: &str = "examples/programming-language/main.veac";
const STDLIB_DOC: &str = "docs/language-reference/standard-library.md";

#[test]
fn component_fragments_are_executable_source_graph_excerpts() {
    let root = workspace_root();
    let documentation = read(&root, DOC);
    let fragments = fenced_blocks(&documentation, "veac,fragment");
    assert_eq!(
        fragments.len(),
        4,
        "{DOC} must keep four component excerpts"
    );

    let inputs = parse_build_input_manifest(&read(
        &root,
        "examples/programming-language/build-inputs.json",
    ))
    .unwrap();
    let built = build_path_with_inputs(&root.join(EXAMPLE), &inputs)
        .unwrap_or_else(|errors| panic!("{EXAMPLE} does not build: {errors}"));
    let source_graph = built
        .sources()
        .values()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    let normalized_graph = normalize(&source_graph);
    for fragment in fragments {
        let normalized = normalize(fragment);
        assert!(
            normalized_graph.contains(&normalized),
            "documented component fragment drifted from {EXAMPLE}:\n{fragment}"
        );
    }
}

#[test]
fn documented_plural_component_program_executes_with_iteration_provenance() {
    let root = workspace_root();
    let documentation = read(&root, AGENT_DOC);
    let programs = fenced_blocks(&documentation, "veac,executable");
    assert_eq!(
        programs.len(),
        1,
        "{AGENT_DOC} must publish one full batch program"
    );
    let built = build_source(programs[0])
        .unwrap_or_else(|errors| panic!("{AGENT_DOC} batch program does not build: {errors}"));
    let sequence = &built.envelope().project.sequences[0];
    let clips = &sequence.tracks[0].clips;
    assert_eq!(clips.len(), 3);
    assert_eq!(
        clips
            .iter()
            .map(|clip| clip
                .authorship
                .as_ref()
                .unwrap()
                .logical_path
                .last()
                .unwrap()
                .as_str())
            .collect::<Vec<_>>(),
        ["first", "second", "third"]
    );
    let iterations = clips[2]
        .authorship
        .as_ref()
        .unwrap()
        .events
        .iter()
        .flat_map(|event| &event.iterations)
        .map(|iteration| iteration.index)
        .collect::<Vec<_>>();
    assert_eq!(iterations, [0]);
    let veac_ir::SequenceAuthorship::Veac { entity, tracks, .. } =
        sequence.authorship.as_ref().unwrap()
    else {
        panic!("expected VEAC authorship")
    };
    assert_eq!(
        entity.events[1].operation.0,
        Op::SequenceWithLayers.opcode()
    );
    let operations = tracks[0]
        .entity
        .events
        .iter()
        .map(|event| event.operation.0)
        .collect::<Vec<_>>();
    assert_eq!(
        &operations[1..],
        [Op::LayerWithItems.opcode(), Op::LayerWithItems.opcode()]
    );
    assert_eq!(
        built
            .envelope()
            .project
            .authorship
            .as_ref()
            .unwrap()
            .entity
            .events[1]
            .operation
            .0,
        Op::ProjectWithSequences.opcode()
    );
}

#[test]
fn standard_library_reference_defers_numeric_identity_to_language_spec() {
    let source = read(&workspace_root(), STDLIB_DOC);
    assert!(
        !source.contains("0x"),
        "{STDLIB_DOC} must not duplicate numeric opcode tables"
    );
    for contract in [
        "559 个 free function 加 23 个 method",
        "582 项 callable inventory",
    ] {
        assert!(
            source.contains(contract),
            "missing standard-library contract: {contract}"
        );
    }
    assert!(source.contains("veac language-spec"));
    assert!(source.contains(&DomainOperationRegistry::standard().digest().to_string()));
}

fn fenced_blocks<'a>(source: &'a str, language: &str) -> Vec<&'a str> {
    let marker = format!("```{language}\n");
    source
        .split(&marker)
        .skip(1)
        .map(|tail| tail.split_once("\n```").expect("closed code fence").0)
        .collect()
}

fn normalize(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn read(root: &std::path::Path, relative: &str) -> String {
    let path = root.join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
