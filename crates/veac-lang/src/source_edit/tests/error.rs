use super::*;

#[test]
fn every_error_has_a_stable_human_readable_message() {
    let errors = [
        SourceEditError::InvalidSchema,
        SourceEditError::UnsupportedSchemaVersion(2),
        SourceEditError::InvalidOperationId("bad".to_owned()),
        SourceEditError::AtomicRequired,
        SourceEditError::EmptyOperations,
        SourceEditError::TooManyPreconditions { limit: 4_096 },
        SourceEditError::TooManyOperations { limit: 4_096 },
        SourceEditError::TooManyTextEdits { limit: 4_096 },
        SourceEditError::ExpressionPayloadTooLarge { limit: 16 },
        SourceEditError::ReplacementPayloadTooLarge { limit: 16 },
        SourceEditError::EditedSourceTooLarge { limit: 16 },
        SourceEditError::SourceEditWorkingSetTooLarge { limit: 48 },
        SourceEditError::SourceEditJsonTooLarge { limit: 64 },
        SourceEditError::SourceEditSizeOverflow,
        SourceEditError::InvalidDigest("bad".to_owned()),
        SourceEditError::EmptySourceGraph,
        SourceEditError::InvalidModulePath("../bad".to_owned()),
        SourceEditError::DuplicateModulePath("main.veac".to_owned()),
        SourceEditError::InvalidNodeId("bad.id".to_owned()),
        SourceEditError::InvalidExpression("empty".to_owned()),
        SourceEditError::IncompatibleExpressionSite,
        SourceEditError::StaleRevision {
            expected: "a".to_owned(),
            actual: "b".to_owned(),
        },
        SourceEditError::PreconditionFailed { index: 1 },
        SourceEditError::InvalidTextRange { start: 2, end: 1 },
        SourceEditError::TextRangeNotUtf8Boundary { offset: 2 },
        SourceEditError::OverlappingTextEdits,
        SourceEditError::ResolvedModuleMismatch {
            expected: "a".to_owned(),
            actual: "b".to_owned(),
        },
    ];
    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(std::error::Error::source(&error).is_none());
    }
}
