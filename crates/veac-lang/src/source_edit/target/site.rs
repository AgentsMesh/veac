use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{BodySite, ExpressionSite, SourceExpressionPath, SourceNodeKind, SourceNodeRef};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum StatementSite {
    BodyStatement { path: SourceExpressionPath },
}

impl StatementSite {
    pub fn accepts(&self, kind: SourceNodeKind) -> bool {
        matches!(kind, SourceNodeKind::Function | SourceNodeKind::Method)
    }

    pub fn is_valid(&self) -> bool {
        let Self::BodyStatement { path } = self;
        path.is_valid() && path.ends_in_local_value()
    }

    pub fn accepts_target(&self, target: &SourceNodeRef) -> bool {
        self.accepts(target.kind())
    }
}

impl ExpressionSite {
    pub fn accepts(&self, kind: SourceNodeKind) -> bool {
        matches!(
            (self, kind),
            (Self::ConstantValue, SourceNodeKind::Constant)
                | (Self::BodyExpression { .. }, SourceNodeKind::Function)
                | (Self::BodyExpression { .. }, SourceNodeKind::Method)
        )
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::ConstantValue => true,
            Self::BodyExpression { path } => path.is_valid(),
        }
    }

    pub fn accepts_target(&self, target: &SourceNodeRef) -> bool {
        self.accepts(target.kind())
    }
}

impl BodySite {
    pub fn accepts(self, kind: SourceNodeKind) -> bool {
        matches!(
            (self, kind),
            (Self::FunctionBody, SourceNodeKind::Function)
                | (Self::MethodBody, SourceNodeKind::Method)
                | (Self::TemporalAnimation { .. }, SourceNodeKind::Temporal)
                | (Self::ComponentAnimation { .. }, SourceNodeKind::Function)
                | (Self::ComponentAnimation { .. }, SourceNodeKind::Method)
        )
    }

    pub fn accepts_target(self, target: &SourceNodeRef) -> bool {
        self.accepts(target.kind())
    }
}
