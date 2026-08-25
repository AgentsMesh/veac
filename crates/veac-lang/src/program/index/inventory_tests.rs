use crate::program::{prepare_source, source_index_json_schema, SourceIndexInventory};
use crate::source_edit::{BodySite, ExpressionSite, SourceNodeRef};
use std::collections::BTreeMap;

pub(super) const SOURCE: &str = r#"fn passthrough(value: time) -> time { value }
const text title = "inventory";
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), "索引测试",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
  project(identifier("inventory"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn inventory_is_stable_complete_and_json_round_trippable() {
    let prepared = prepare_source(SOURCE).unwrap();
    let inventory = prepared.source_inventory().unwrap();
    assert_eq!(inventory.revision, prepared.source_revision().unwrap());
    assert_eq!(inventory.schema, super::SOURCE_INDEX_SCHEMA);
    assert_eq!(inventory.schema_version, super::SOURCE_INDEX_SCHEMA_VERSION);
    assert!(inventory.build_inputs.is_empty());
    assert!(inventory
        .nodes
        .windows(2)
        .all(|pair| pair[0].target < pair[1].target));
    let body = node(
        &inventory,
        SourceNodeRef::function("main.veac", "passthrough"),
    )
    .bodies
    .iter()
    .find(|value| value.site == BodySite::FunctionBody)
    .unwrap();
    assert_eq!(body.source, "{ value }");
    assert_eq!(&SOURCE[body.range.start..body.range.end], body.source);
    let json = serde_json::to_string(&inventory).unwrap();
    assert_eq!(
        serde_json::from_str::<SourceIndexInventory>(&json).unwrap(),
        inventory
    );
    let schema = source_index_json_schema().unwrap().to_string();
    for value in [
        "constant_value",
        "body_statement",
        "statements",
        "function_body",
        "method_body",
        "temporal_animation",
        "item",
        "method",
        "declarations",
        "struct_declaration",
        "struct_field_declaration",
        "enum_declaration",
        "enum_variant_declaration",
        "enum_variant_field_declaration",
    ] {
        assert!(
            schema.contains(value),
            "source-index schema omitted {value}"
        );
    }
    assert!(schema.contains(r#""const":"https://veac.dev/schemas/source-index""#));
    assert!(schema.contains(&format!(
        r#""const":{}"#,
        super::SOURCE_INDEX_SCHEMA_VERSION
    )));
    assert!(schema.contains("build_inputs"));
}

#[test]
fn inventory_preserves_function_body_source_and_range() {
    let inventory = prepare_source(SOURCE).unwrap().source_inventory().unwrap();
    let body = node(&inventory, SourceNodeRef::function("main.veac", "main"))
        .bodies
        .iter()
        .find(|value| value.site == BodySite::FunctionBody)
        .unwrap();
    assert!(body.source.contains("project(identifier(\"inventory\")"));
    assert_eq!(&SOURCE[body.range.start..body.range.end], body.source);
}

#[test]
fn direct_index_build_rejects_invalid_graphs_and_ambiguous_targets() {
    let invalid = BTreeMap::from([(".veac-source.lock".to_owned(), "module {}".to_owned())]);
    assert_eq!(
        super::SourceIndex::build_snapshot(&invalid)
            .unwrap_err()
            .as_slice()[0]
            .code,
        "SOURCE_GRAPH_REVISION"
    );

    let duplicate = BTreeMap::from([(
        "main.veac".to_owned(),
        "const int same = 1; const int same = 2;".to_owned(),
    )]);
    assert_eq!(
        super::SourceIndex::build_snapshot(&duplicate)
            .unwrap_err()
            .as_slice()[0]
            .code,
        "SOURCE_INDEX_AMBIGUOUS_TARGET"
    );
}

#[test]
fn constant_ranges_separate_complete_declarations_from_expressions() {
    let entry = "const time base =   1s + 2s   ;";
    let module = "module {\n  export const time public = 3s ;\n}";
    let sources = BTreeMap::from([
        ("main.veac".to_owned(), entry.to_owned()),
        ("timing.veac".to_owned(), module.to_owned()),
    ]);
    let index = super::test_support::index(&sources);
    let revision = super::test_revision(&index);
    let inventory = index.inventory(&revision).unwrap();

    assert_constant_ranges(&inventory, "main.veac", "base", entry, entry, "1s + 2s");
    assert_constant_ranges(
        &inventory,
        "timing.veac",
        "public",
        module,
        "export const time public = 3s ;",
        "3s",
    );
}

fn assert_constant_ranges(
    inventory: &SourceIndexInventory,
    module: &str,
    name: &str,
    authored: &str,
    declaration: &str,
    expression: &str,
) {
    let target = SourceNodeRef::constant(module, name);
    let module = inventory
        .modules
        .iter()
        .find(|value| value.module == module)
        .unwrap();
    let declaration_site = module
        .declarations
        .iter()
        .find(|value| value.target == target)
        .unwrap();
    assert_eq!(declaration_site.source, declaration);
    assert_eq!(
        &authored[declaration_site.range.start..declaration_site.range.end],
        declaration
    );
    assert_eq!(declaration_site.source.matches(';').count(), 1);
    let expression_site = node(inventory, target)
        .expressions
        .iter()
        .find(|value| value.site == ExpressionSite::ConstantValue)
        .unwrap();
    assert_eq!(expression_site.source, expression);
    assert_eq!(
        &authored[expression_site.range.start..expression_site.range.end],
        expression
    );
}

fn node(inventory: &SourceIndexInventory, target: SourceNodeRef) -> &super::SourceIndexNode {
    inventory
        .nodes
        .iter()
        .find(|value| value.target == target)
        .unwrap()
}
