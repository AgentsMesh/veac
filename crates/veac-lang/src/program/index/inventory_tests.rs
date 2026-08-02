use crate::program::{compile_source, source_index_json_schema, SourceIndexInventory};
use crate::source_edit::{ExpressionSite, SourceNodeRef};

const SOURCE: &str = r#"const text title = "inventory";
component sequence card {
  param time duration default 1s + 500ms;
  body {}
}
instance sequence opener from card { bind duration 2s; }
project inventory {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main {
    layer visual content {
      item title {
        source text { content ${title}; }
        record { at 0s; duration 2s; }
      }
    }
  }
}"#;

#[test]
fn inventory_is_stable_complete_and_json_round_trippable() {
    let index = compile_source(SOURCE).unwrap().source_index().unwrap();
    let inventory = index.inventory();
    assert_eq!(inventory.schema, super::SOURCE_INDEX_SCHEMA);
    assert_eq!(inventory.schema_version, super::SOURCE_INDEX_SCHEMA_VERSION);
    assert!(inventory
        .nodes
        .windows(2)
        .all(|pair| pair[0].target < pair[1].target));
    assert!(
        node(&inventory, SourceNodeRef::project("main.veac", "inventory"))
            .expressions
            .is_empty()
    );
    assert_source(
        &inventory,
        SourceNodeRef::component("main.veac", "card"),
        ExpressionSite::ComponentParameterDefault {
            parameter: "duration".into(),
        },
        "1s + 500ms",
    );
    assert_source(
        &inventory,
        SourceNodeRef::component_instance("main.veac", "opener"),
        ExpressionSite::ComponentInstanceArgument {
            parameter: "duration".into(),
        },
        "2s",
    );
    let json = serde_json::to_string(&inventory).unwrap();
    assert_eq!(
        serde_json::from_str::<SourceIndexInventory>(&json).unwrap(),
        inventory
    );
    let schema = source_index_json_schema().unwrap().to_string();
    for value in [
        "preset_audio_processor",
        "preset_audio_eq_band",
        "preset_audio_processor_field",
        "preset_audio_eq_band_field",
        "frequency",
        "gain",
        "q",
    ] {
        assert!(
            schema.contains(value),
            "source-index schema omitted {value}"
        );
    }
}

#[test]
fn inventory_preserves_current_project_expression_and_range() {
    let inventory = compile_source(SOURCE)
        .unwrap()
        .source_index()
        .unwrap()
        .inventory();
    let target = SourceNodeRef::item("main.veac", "inventory", "main", "content", "title");
    let expression = node(&inventory, target)
        .expressions
        .iter()
        .find(|value| value.site == ExpressionSite::TextContent)
        .unwrap();
    assert_eq!(expression.source, "${title}");
    assert_eq!(
        &SOURCE[expression.range.start..expression.range.end],
        "${title}"
    );
}

fn node(inventory: &SourceIndexInventory, target: SourceNodeRef) -> &super::SourceIndexNode {
    inventory
        .nodes
        .iter()
        .find(|value| value.target == target)
        .unwrap()
}

fn assert_source(
    inventory: &SourceIndexInventory,
    target: SourceNodeRef,
    site: ExpressionSite,
    expected: &str,
) {
    let expression = node(inventory, target)
        .expressions
        .iter()
        .find(|value| value.site == site)
        .unwrap();
    assert_eq!(expression.source, expected);
}
