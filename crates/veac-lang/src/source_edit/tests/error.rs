use super::*;

#[test]
fn every_error_has_a_stable_human_readable_message() {
    let errors = [
        SourceEditError::InvalidSchema,
        SourceEditError::UnsupportedSchemaVersion(1),
        SourceEditError::InvalidOperationId("bad".to_owned()),
        SourceEditError::AtomicRequired,
        SourceEditError::EmptyOperations,
        SourceEditError::TooManyPreconditions { limit: 4_096 },
        SourceEditError::TooManyOperations { limit: 4_096 },
        SourceEditError::TooManyTextEdits { limit: 4_096 },
        SourceEditError::FragmentPayloadTooLarge { limit: 16 },
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
        SourceEditError::InvalidExpressionPath,
        SourceEditError::InvalidExpression("empty".to_owned()),
        SourceEditError::IncompatibleExpressionSite,
        SourceEditError::InvalidStatementPath,
        SourceEditError::InvalidStatement("empty".to_owned()),
        SourceEditError::IncompatibleStatementSite,
        SourceEditError::InvalidBody("empty".to_owned()),
        SourceEditError::IncompatibleBodySite,
        SourceEditError::InvalidDeclaration("empty".to_owned()),
        SourceEditError::IncompatibleDeclarationSite,
        SourceEditError::InvalidTopLevelDeclaration("empty".to_owned()),
        SourceEditError::IncompatibleTopLevelDeclarationTarget,
        SourceEditError::InvalidImport("empty".to_owned()),
        SourceEditError::AnchorModuleMismatch {
            module: "main.veac".to_owned(),
            anchor_module: "other.veac".to_owned(),
        },
        SourceEditError::StructuralOperationRequiresIndex,
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
