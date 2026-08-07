use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Interpolation, RationalTime};

use super::{
    TemporalBinaryOperation, TemporalCompareOperation, TemporalInputId, TemporalNodeId,
    TemporalType, TemporalUnaryOperation, TemporalValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemporalCurvePosition {
    Scalar { value: f64 },
    Time { value: RationalTime },
}

impl TemporalCurvePosition {
    pub const fn value_type(self) -> TemporalType {
        match self {
            Self::Scalar { .. } => TemporalType::Scalar,
            Self::Time { .. } => TemporalType::Time,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalCurveKey {
    pub position: TemporalCurvePosition,
    pub value: TemporalValue,
    pub interpolation: Interpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalVectorAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalPointAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalRectField {
    X,
    Y,
    Width,
    Height,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemporalNodeKind {
    Literal {
        value: TemporalValue,
    },
    Input {
        input_id: TemporalInputId,
    },
    Unary {
        operation: TemporalUnaryOperation,
        operand: TemporalNodeId,
    },
    Binary {
        operation: TemporalBinaryOperation,
        left: TemporalNodeId,
        right: TemporalNodeId,
    },
    Compare {
        operation: TemporalCompareOperation,
        left: TemporalNodeId,
        right: TemporalNodeId,
    },
    Select {
        condition: TemporalNodeId,
        when_true: TemporalNodeId,
        when_false: TemporalNodeId,
    },
    CurveSample {
        input: TemporalNodeId,
        keys: Vec<TemporalCurveKey>,
    },
    ComposeVec2 {
        x: TemporalNodeId,
        y: TemporalNodeId,
    },
    ProjectVec2 {
        value: TemporalNodeId,
        axis: TemporalVectorAxis,
    },
    ComposePoint {
        x: TemporalNodeId,
        y: TemporalNodeId,
    },
    ProjectPoint {
        value: TemporalNodeId,
        axis: TemporalPointAxis,
    },
    ComposeRect {
        x: TemporalNodeId,
        y: TemporalNodeId,
        width: TemporalNodeId,
        height: TemporalNodeId,
    },
    ProjectRect {
        value: TemporalNodeId,
        field: TemporalRectField,
    },
    ComposeColor {
        red: TemporalNodeId,
        green: TemporalNodeId,
        blue: TemporalNodeId,
        alpha: TemporalNodeId,
    },
    ProjectColor {
        value: TemporalNodeId,
        channel: TemporalColorChannel,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalNode {
    pub id: TemporalNodeId,
    pub value_type: TemporalType,
    pub kind: TemporalNodeKind,
    pub provenance_id: Option<super::TemporalProvenanceId>,
}
