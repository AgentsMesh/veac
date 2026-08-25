use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_SOURCE_EXPRESSION_PATH_DEPTH: usize = 128;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SourceExpressionPath {
    #[schemars(length(min = 1, max = MAX_SOURCE_EXPRESSION_PATH_DEPTH))]
    pub steps: Vec<SourceExpressionStep>,
}

impl SourceExpressionPath {
    pub fn new(steps: Vec<SourceExpressionStep>) -> Self {
        Self { steps }
    }

    pub fn is_valid(&self) -> bool {
        !self.steps.is_empty()
            && self.steps.len() <= MAX_SOURCE_EXPRESSION_PATH_DEPTH
            && self.steps.iter().all(SourceExpressionStep::is_valid)
    }

    pub(crate) fn ends_in_local_value(&self) -> bool {
        matches!(
            self.steps.last(),
            Some(SourceExpressionStep::LocalValue { .. })
        )
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceLocalOperation {
    Let,
    Var,
    Set,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "step", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceExpressionStep {
    LocalValue {
        operation: SourceLocalOperation,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        binding: String,
        ordinal: u32,
    },
    BlockResult,
    UnaryOperand,
    BinaryLeft,
    BinaryRight,
    RangeStart,
    RangeEnd,
    RangeStep,
    ClosureBody,
    IterationSource,
    IterationBody,
    CallCallee,
    CallArgument {
        ordinal: u32,
    },
    NamedCallArgument {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        name: String,
    },
    FieldReceiver,
    NominalField {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        field: String,
    },
    MatchScrutinee,
    MatchArm {
        pattern: String,
    },
    TemporalArgument {
        ordinal: u32,
    },
    TemporalBody,
    ListItem {
        ordinal: u32,
    },
    MapKey {
        ordinal: u32,
    },
    MapValue {
        ordinal: u32,
    },
    TupleItem {
        ordinal: u32,
    },
    IfCondition,
    IfThen,
    IfElse,
}

impl SourceExpressionStep {
    fn is_valid(&self) -> bool {
        match self {
            Self::LocalValue { binding, .. }
            | Self::NominalField { field: binding }
            | Self::NamedCallArgument { name: binding } => crate::name::is_name(binding),
            Self::MatchArm { pattern } => valid_pattern(pattern),
            _ => true,
        }
    }
}

fn valid_pattern(pattern: &str) -> bool {
    pattern == "_"
        || pattern
            .split('.')
            .all(|segment| !segment.is_empty() && crate::name::is_name(segment))
}
