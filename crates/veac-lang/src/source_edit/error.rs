use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceEditError {
    InvalidSchema,
    UnsupportedSchemaVersion(u32),
    InvalidOperationId(String),
    AtomicRequired,
    EmptyOperations,
    TooManyPreconditions { limit: usize },
    TooManyOperations { limit: usize },
    TooManyTextEdits { limit: usize },
    ExpressionPayloadTooLarge { limit: usize },
    ReplacementPayloadTooLarge { limit: usize },
    EditedSourceTooLarge { limit: usize },
    SourceEditWorkingSetTooLarge { limit: usize },
    SourceEditJsonTooLarge { limit: usize },
    SourceEditSizeOverflow,
    InvalidDigest(String),
    EmptySourceGraph,
    InvalidModulePath(String),
    DuplicateModulePath(String),
    InvalidNodeId(String),
    InvalidExpression(String),
    IncompatibleExpressionSite,
    StaleRevision { expected: String, actual: String },
    PreconditionFailed { index: usize },
    InvalidTextRange { start: usize, end: usize },
    TextRangeNotUtf8Boundary { offset: usize },
    OverlappingTextEdits,
    ResolvedModuleMismatch { expected: String, actual: String },
}

impl fmt::Display for SourceEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSchema => formatter.write_str("invalid source-edit schema"),
            Self::UnsupportedSchemaVersion(value) => {
                write!(formatter, "unsupported source-edit schema version {value}")
            }
            Self::InvalidOperationId(value) => write!(formatter, "invalid operation ID {value:?}"),
            Self::AtomicRequired => formatter.write_str("source-edit batches must be atomic"),
            Self::EmptyOperations => {
                formatter.write_str("source-edit batches must contain an operation")
            }
            Self::TooManyPreconditions { limit } => {
                write!(formatter, "source-edit batch exceeds {limit} preconditions")
            }
            Self::TooManyOperations { limit } => {
                write!(formatter, "source-edit batch exceeds {limit} operations")
            }
            Self::TooManyTextEdits { limit } => {
                write!(formatter, "source edit exceeds {limit} text replacements")
            }
            Self::ExpressionPayloadTooLarge { limit } => write!(
                formatter,
                "source-edit expression payload exceeds {limit} bytes"
            ),
            Self::ReplacementPayloadTooLarge { limit } => write!(
                formatter,
                "source-edit replacement payload exceeds {limit} bytes"
            ),
            Self::EditedSourceTooLarge { limit } => {
                write!(formatter, "edited source exceeds {limit} bytes")
            }
            Self::SourceEditWorkingSetTooLarge { limit } => write!(
                formatter,
                "source-edit string working set exceeds {limit} bytes"
            ),
            Self::SourceEditJsonTooLarge { limit } => {
                write!(formatter, "source-edit JSON exceeds {limit} bytes")
            }
            Self::SourceEditSizeOverflow => {
                formatter.write_str("source-edit size arithmetic overflowed")
            }
            Self::InvalidDigest(value) => write!(formatter, "invalid SHA-256 digest {value:?}"),
            Self::EmptySourceGraph => formatter.write_str("source graph must not be empty"),
            Self::InvalidModulePath(value) => write!(formatter, "invalid module {value:?}"),
            Self::DuplicateModulePath(value) => {
                write!(formatter, "duplicate module {value:?}")
            }
            Self::InvalidNodeId(value) => write!(formatter, "invalid source node ID {value:?}"),
            Self::InvalidExpression(value) => write!(formatter, "invalid expression: {value}"),
            Self::IncompatibleExpressionSite => {
                formatter.write_str("expression site is incompatible with the target node kind")
            }
            Self::StaleRevision { expected, actual } => {
                write!(
                    formatter,
                    "stale source revision: expected {expected}, got {actual}"
                )
            }
            Self::PreconditionFailed { index } => {
                write!(formatter, "source-edit precondition {index} failed")
            }
            Self::InvalidTextRange { start, end } => {
                write!(formatter, "invalid text range {start}..{end}")
            }
            Self::TextRangeNotUtf8Boundary { offset } => {
                write!(
                    formatter,
                    "text edit offset {offset} is not a UTF-8 boundary"
                )
            }
            Self::OverlappingTextEdits => formatter.write_str("text edits overlap"),
            Self::ResolvedModuleMismatch { expected, actual } => write!(
                formatter,
                "resolved edit targets module {actual:?}, expected {expected:?}"
            ),
        }
    }
}

impl std::error::Error for SourceEditError {}
