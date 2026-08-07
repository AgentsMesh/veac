#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceEditError {
    InvalidSchema,
    UnsupportedSchemaVersion(u32),
    InvalidOperationId(String),
    AtomicRequired,
    EmptyOperations,
    TooManyPreconditions {
        limit: usize,
    },
    TooManyOperations {
        limit: usize,
    },
    TooManyTextEdits {
        limit: usize,
    },
    FragmentPayloadTooLarge {
        limit: usize,
    },
    ReplacementPayloadTooLarge {
        limit: usize,
    },
    EditedSourceTooLarge {
        limit: usize,
    },
    SourceEditWorkingSetTooLarge {
        limit: usize,
    },
    SourceEditJsonTooLarge {
        limit: usize,
    },
    SourceEditSizeOverflow,
    InvalidDigest(String),
    EmptySourceGraph,
    InvalidModulePath(String),
    DuplicateModulePath(String),
    InvalidNodeId(String),
    InvalidExpressionPath,
    InvalidExpression(String),
    IncompatibleExpressionSite,
    InvalidStatementPath,
    InvalidStatement(String),
    IncompatibleStatementSite,
    InvalidBody(String),
    IncompatibleBodySite,
    InvalidDeclaration(String),
    IncompatibleDeclarationSite,
    InvalidTopLevelDeclaration(String),
    IncompatibleTopLevelDeclarationTarget,
    InvalidImport(String),
    AnchorModuleMismatch {
        module: String,
        anchor_module: String,
    },
    StructuralOperationRequiresIndex,
    StaleRevision {
        expected: String,
        actual: String,
    },
    PreconditionFailed {
        index: usize,
    },
    InvalidTextRange {
        start: usize,
        end: usize,
    },
    TextRangeNotUtf8Boundary {
        offset: usize,
    },
    OverlappingTextEdits,
    ResolvedModuleMismatch {
        expected: String,
        actual: String,
    },
}

impl std::error::Error for SourceEditError {}

#[path = "error/display.rs"]
mod display;
