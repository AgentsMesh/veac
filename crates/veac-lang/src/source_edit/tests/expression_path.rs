use super::*;

#[test]
fn nested_expression_paths_are_closed_bounded_and_agent_readable() {
    let path = SourceExpressionPath::new(vec![
        SourceExpressionStep::LocalValue {
            operation: SourceLocalOperation::Let,
            binding: "clips".to_owned(),
            ordinal: 0,
        },
        SourceExpressionStep::CallArgument { ordinal: 1 },
        SourceExpressionStep::ClosureBody,
        SourceExpressionStep::BlockResult,
    ]);
    assert!(path.is_valid());
    let json = serde_json::to_value(&path).unwrap();
    assert_eq!(json["steps"][0]["step"], "local_value");
    assert_eq!(json["steps"][0]["operation"], "let");
    assert_eq!(
        serde_json::from_value::<SourceExpressionPath>(json).unwrap(),
        path
    );
}

#[test]
fn invalid_nested_paths_fail_before_source_lookup() {
    for path in [
        SourceExpressionPath::new(Vec::new()),
        SourceExpressionPath::new(vec![SourceExpressionStep::NominalField {
            field: "not a name".to_owned(),
        }]),
        SourceExpressionPath::new(vec![SourceExpressionStep::NamedCallArgument {
            name: "not a name".to_owned(),
        }]),
        SourceExpressionPath::new(vec![SourceExpressionStep::MatchArm {
            pattern: "Choice..Broken".to_owned(),
        }]),
        SourceExpressionPath::new(vec![SourceExpressionStep::BlockResult; 129]),
    ] {
        let mut value = batch();
        value.operations = vec![SourceEditOperation::SetExpression {
            target: SourceNodeRef::function("timeline/main.veac", "duration"),
            site: ExpressionSite::BodyExpression { path },
            expression: ExpressionSource {
                source: "1s".to_owned(),
            },
        }];
        assert_eq!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidExpressionPath)
        );
    }
}

#[test]
fn named_call_paths_roundtrip_as_closed_schema_values() {
    let path = SourceExpressionPath::new(vec![SourceExpressionStep::NamedCallArgument {
        name: "duration".to_owned(),
    }]);
    assert!(path.is_valid());
    let json = serde_json::to_value(&path).unwrap();
    assert_eq!(json["steps"][0]["step"], "named_call_argument");
    assert_eq!(json["steps"][0]["name"], "duration");
    assert_eq!(
        serde_json::from_value::<SourceExpressionPath>(json).unwrap(),
        path
    );
}
