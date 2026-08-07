use std::fs;
use std::path::PathBuf;

#[test]
fn nominal_reference_pins_the_executable_contract() {
    let reference = read("docs/language-reference/programming-nominal.md");
    for token in [
        "TypeDefinitionDigest",
        "exhaustive `Match`",
        "resolved Typed HIR",
        "GraphEmit",
        "`MethodBody`",
        "`method_body`",
    ] {
        assert!(
            reference.contains(token),
            "nominal reference is missing {token}"
        );
    }
    assert!(read("docs/language-reference/programming.md").contains("programming-nominal.md"));
    assert!(read("docs/language-reference/README.md").contains("programming-nominal.md"));
}

#[test]
fn checked_in_example_exercises_nominal_values_and_methods_visibly() {
    let module = read("examples/programming-language/brand.veac");
    let entry = read("examples/programming-language/main.veac");
    for token in [
        "export struct Card",
        "export enum Mood",
        "impl Card @presentation",
        "match self.mood",
        "card.background()",
        "结构与方法",
        "枚举与控制流",
    ] {
        assert!(
            module.contains(token),
            "programming module is missing {token}"
        );
    }
    for token in [
        "list<brand.Card>",
        "brand.first_card(duration)",
        "brand.second_card(duration, accent)",
        "map(cards(duration, accent_color)",
        "fn(card: brand.Card)",
        "card.title(\"：\")",
        ".with_items(backdrops)",
        ".with_items(copies)",
        ".with_layers([backdrop_layer, title_layer])",
        ".with_resources([font])",
        ".with_sequences([timeline])",
        ".with_deliveries([output])",
    ] {
        assert!(
            entry.contains(token),
            "programming example is missing {token}"
        );
    }
}

#[test]
fn source_editing_reference_pins_nominal_index_v8() {
    let source = read("docs/language-reference/source-editing.md");
    assert!(source.contains("`source-index` v8"));
    let nominal = read("docs/language-reference/programming-nominal.md");
    assert!(nominal.contains("\"kind\": \"method\""));
    assert!(nominal.contains("{ \"type\": \"method_body\" }"));
}

fn read(relative: &str) -> String {
    fs::read_to_string(workspace_root().join(relative)).unwrap()
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
