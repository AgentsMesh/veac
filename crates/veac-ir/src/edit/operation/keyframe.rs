use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "target", rename_all = "snake_case", deny_unknown_fields)]
pub enum NumberCurveTarget {
    VisualOpacity,
    VisualRotationDegrees,
    MaskRotationDegrees {
        mask_index: u32,
    },
    MaskFeatherPixels {
        mask_index: u32,
    },
    MaskExpansionPixels {
        mask_index: u32,
    },
    AudioGain,
    AudioPan,
    TextReveal,
    TextHighlightProgress,
    TextOpacity,
    TextRotationDegrees,
    EffectParameter {
        effect_id: EffectId,
        parameter: EffectParameter,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PointCurveTarget {
    VisualPosition,
    TextPositionOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "target", rename_all = "snake_case", deny_unknown_fields)]
pub enum Vec2CurveTarget {
    VisualScale,
    TextScale,
    MaskPosition { mask_index: u32 },
    MaskScale { mask_index: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RectCurveTarget {
    VisualCrop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum KeyframeEdit {
    UpsertNumber {
        clip_id: ItemId,
        target: NumberCurveTarget,
        keyframe: Keyframe<f64>,
    },
    UpsertPoint {
        clip_id: ItemId,
        target: PointCurveTarget,
        keyframe: Keyframe<Point>,
    },
    UpsertVec2 {
        clip_id: ItemId,
        target: Vec2CurveTarget,
        keyframe: Keyframe<Vec2>,
    },
    UpsertRect {
        clip_id: ItemId,
        target: RectCurveTarget,
        keyframe: Keyframe<Rect>,
    },
    Remove {
        clip_id: ItemId,
        keyframe_id: KeyframeId,
    },
    Move {
        clip_id: ItemId,
        keyframe_id: KeyframeId,
        time: RationalTime,
    },
}
