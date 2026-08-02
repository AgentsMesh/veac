use super::local_declarations;
use crate::authoring::Span;

#[test]
fn every_core_timeline_declaration_shape_has_an_origin() {
    let source = r#"layer visual @layer {
  item @item {
    mapping curve { key @key { at 0s; } }
    modifiers { effect @modifier { type video.blur; } }
  }
}
relation group @relation { members { item @item; } }
apply @apply {
  pipeline { stage effect @stage { type video.blur; } }
  mix { mask @mask { shape circle; } }
}"#;
    let span = Span {
        start: 0,
        end: source.len(),
    };
    let declarations = local_declarations("component.veac", source, span)
        .unwrap()
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    assert_eq!(
        declarations,
        ["layer", "item", "key", "modifier", "relation", "apply", "stage", "mask"]
    );
}

#[test]
fn references_are_not_reported_as_declarations() {
    let source = "apply @grade { scope layer @target; }";
    let span = Span {
        start: 0,
        end: source.len(),
    };
    let declarations = local_declarations("component.veac", source, span).unwrap();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].0, "grade");
}
