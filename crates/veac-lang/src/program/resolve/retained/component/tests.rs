use std::collections::BTreeMap;

use crate::program::model::ComponentKey;
use crate::program::parser;

use super::{logical_bytes, ENTRY_BYTES};

#[test]
fn every_captured_component_binding_has_a_deterministic_logical_charge() {
    let source = r#"component sequence card { body {} }
project retained {
  settings {
    timebase 1/1000; canvas 1px by 1px;
    frame-rate 1fps; sample-rate 8000hz;
  }
  entry sequence main; sequence main {}
}"#;
    let file = parser::parse("main.veac", source).unwrap();
    let empty = logical_bytes(&file, &BTreeMap::new()).unwrap();
    let alias = "shared.leaf";
    let key = ComponentKey {
        path: "shared.veac".to_owned(),
        name: "leaf".to_owned(),
    };
    let captured = BTreeMap::from([(alias.to_owned(), key.clone())]);
    let with_capture = logical_bytes(&file, &captured).unwrap();
    assert_eq!(
        with_capture - empty,
        ENTRY_BYTES + alias.len() + key.path.len() + key.name.len()
    );
}
