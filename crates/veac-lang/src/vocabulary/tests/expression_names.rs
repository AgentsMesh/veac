use std::collections::BTreeSet;

use serde_json::json;

use super::super::{
    accepts_expression_name, language_spec, ExpressionNameKind, IdentifierPolicy, LanguageLayer,
    VocabularyCategory,
};

#[test]
fn expression_control_names_are_derived_from_owned_grammar_uses() {
    let vocabulary = language_spec().vocabulary;
    let controls = vocabulary
        .entries
        .iter()
        .filter(|entry| {
            entry.uses.iter().any(|usage| {
                usage.layer == LanguageLayer::ExecutableExpression
                    && usage.category == VocabularyCategory::ContextualControl
            })
        })
        .map(|entry| entry.spelling.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        controls,
        BTreeSet::from([
            "animate".into(),
            "by".into(),
            "else".into(),
            "effect".into(),
            "fn".into(),
            "for".into(),
            "if".into(),
            "in".into(),
            "let".into(),
            "match".into(),
            "set".into(),
            "var".into(),
        ])
    );
    for kind in ExpressionNameKind::ALL {
        for control in &controls {
            assert!(!accepts_expression_name(control, kind));
            assert_eq!(
                vocabulary.lookup(control).unwrap().identifier_policy,
                IdentifierPolicy::Allowed
            );
            assert!(crate::name::is_name(control));
        }
        assert!(accepts_expression_name("render_title", kind));
    }
}

#[test]
fn expression_name_kinds_have_an_exact_closed_json_schema() {
    let schema = serde_json::to_value(schemars::schema_for!(ExpressionNameKind)).unwrap();
    let published = schema
        .get("enum")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    let expected = ExpressionNameKind::ALL
        .into_iter()
        .map(|kind| serde_json::to_value(kind).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(published, &expected);
    assert_eq!(
        published,
        &vec![json!("function"), json!("parameter"), json!("local")]
    );
}

#[test]
fn global_literals_and_invalid_shapes_remain_rejected() {
    for kind in ExpressionNameKind::ALL {
        for spelling in ["", "true", "false", "bad name", "标题"] {
            assert!(!accepts_expression_name(spelling, kind), "{spelling}");
        }
    }
}
