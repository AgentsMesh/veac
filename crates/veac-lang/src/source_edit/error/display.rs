use std::fmt;

use super::SourceEditError;

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
            Self::FragmentPayloadTooLarge { limit } => write!(
                formatter,
                "source-edit fragment payload exceeds {limit} bytes"
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
            Self::DuplicateModulePath(value) => write!(formatter, "duplicate module {value:?}"),
            Self::InvalidNodeId(value) => write!(formatter, "invalid source node ID {value:?}"),
            Self::InvalidExpressionPath => {
                formatter.write_str("invalid nested source expression path")
            }
            Self::InvalidExpression(value) => write!(formatter, "invalid expression: {value}"),
            Self::IncompatibleExpressionSite => {
                formatter.write_str("expression site is incompatible with the target node kind")
            }
            Self::InvalidStatementPath => {
                formatter.write_str("invalid nested source statement path")
            }
            Self::InvalidStatement(value) => write!(formatter, "invalid statement: {value}"),
            Self::IncompatibleStatementSite => {
                formatter.write_str("statement site is incompatible with the target node kind")
            }
            Self::InvalidBody(value) => write!(formatter, "invalid body: {value}"),
            Self::IncompatibleBodySite => {
                formatter.write_str("body site is incompatible with the target node kind")
            }
            Self::InvalidDeclaration(value) => write!(formatter, "invalid declaration: {value}"),
            Self::IncompatibleDeclarationSite => {
                formatter.write_str("declaration site is incompatible with the target node kind")
            }
            Self::InvalidTopLevelDeclaration(value) => {
                write!(formatter, "invalid top-level declaration: {value}")
            }
            Self::IncompatibleTopLevelDeclarationTarget => {
                formatter.write_str("target is not a removable top-level declaration")
            }
            Self::InvalidImport(value) => write!(formatter, "invalid import: {value}"),
            Self::UnboundSourceIndex => {
                formatter.write_str("source index is not bound to a prepared source graph")
            }
            Self::AnchorModuleMismatch {
                module,
                anchor_module,
            } => write!(
                formatter,
                "source anchor belongs to module {anchor_module:?}, expected {module:?}"
            ),
            Self::StructuralOperationRequiresIndex => {
                formatter.write_str("structural source edits require a semantic source index")
            }
            Self::StaleRevision { expected, actual } => write!(
                formatter,
                "stale source revision: expected {expected}, got {actual}"
            ),
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
