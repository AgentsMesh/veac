use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;
use veac_ir::{AnnotationId, EditOperation, MaterialId, MulticamGroupId};

use crate::{ProviderResult, StemKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransformComponent {
    Position,
    Scale,
    Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisEvidenceKind {
    Language,
    SceneBoundary,
    Scene,
    Beat,
    Silence,
    Filler,
    FillerDecision,
    Highlight,
    HighlightDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationBinding {
    pub operation_index: u32,
    pub operation_hash: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchControlEvidence {
    pub control: String,
    pub apply_stage_id: veac_ir::ApplyStageId,
    pub effect_id: veac_ir::EffectId,
    pub effect_parameter: String,
    pub keyframe_id_prefix: String,
    pub sample_indices: Vec<u32>,
}

impl OperationBinding {
    pub(crate) fn new(operation_index: u32, operation: &EditOperation) -> ProviderResult<Self> {
        let bytes = serde_json_canonicalizer::to_vec(operation)?;
        Ok(Self {
            operation_index,
            operation_hash: ContentDigest::sha256(&bytes),
        })
    }

    pub(crate) fn matches(&self, operation: &EditOperation) -> bool {
        self.operation_hash.validate().is_ok()
            && serde_json_canonicalizer::to_vec(operation)
                .map(|bytes| self.operation_hash == ContentDigest::sha256(&bytes))
                .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProposalEvidence {
    AsrSegment {
        operation: OperationBinding,
        segment_id: String,
    },
    TranslationUnit {
        operation: OperationBinding,
        unit_id: String,
    },
    AnalysisAnnotation {
        operation: OperationBinding,
        kind: AnalysisEvidenceKind,
        result_index: u32,
        annotation_id: AnnotationId,
    },
    ArtifactMaterial {
        operation: OperationBinding,
        artifact_key: ContentDigest,
        artifact_role: String,
        material_id: MaterialId,
    },
    GeneratedAudioClip {
        operation: OperationBinding,
        artifact_key: ContentDigest,
        artifact_role: String,
        material_id: MaterialId,
    },
    ReplacedMedia {
        operation: OperationBinding,
        artifact_key: ContentDigest,
        artifact_role: String,
        material_id: MaterialId,
    },
    SeparatedStemClip {
        operation: OperationBinding,
        artifact_key: ContentDigest,
        artifact_role: String,
        material_id: MaterialId,
        kind: StemKind,
        label: String,
    },
    ArtifactVisualClip {
        operation: OperationBinding,
        artifact_key: ContentDigest,
        artifact_role: String,
        material_id: MaterialId,
    },
    AppliedTrackMatte {
        operation: OperationBinding,
        artifact_key: ContentDigest,
        artifact_role: String,
        matte_clip_id: veac_ir::ItemId,
    },
    RetouchApply {
        operation: OperationBinding,
        target_clip_id: veac_ir::ItemId,
        apply_id: veac_ir::ApplyId,
        apply_stage_ids: Vec<veac_ir::ApplyStageId>,
        time: super::ClipTimeBinding,
        timebase: u32,
        controls: Vec<RetouchControlEvidence>,
    },
    TransformSamples {
        operation: OperationBinding,
        sample_indices: Vec<u32>,
        component: TransformComponent,
    },
    CropSamples {
        operation: OperationBinding,
        sample_indices: Vec<u32>,
        time: super::ClipTimeBinding,
        timebase: u32,
        keyframe_id_prefix: String,
    },
    ColorPipeline {
        operation: OperationBinding,
    },
    ColorAdjustEffect {
        operation: OperationBinding,
    },
    MulticamSync {
        operation: OperationBinding,
        group_id: MulticamGroupId,
        angle_indices: Vec<u32>,
    },
}

impl ProposalEvidence {
    pub(crate) fn operation_index(&self) -> u32 {
        self.operation().operation_index
    }

    pub(crate) fn matches_operation(&self, value: &EditOperation) -> bool {
        self.operation().matches(value)
    }

    fn operation(&self) -> &OperationBinding {
        match self {
            Self::AsrSegment { operation, .. }
            | Self::TranslationUnit { operation, .. }
            | Self::AnalysisAnnotation { operation, .. }
            | Self::ArtifactMaterial { operation, .. }
            | Self::GeneratedAudioClip { operation, .. }
            | Self::ReplacedMedia { operation, .. }
            | Self::SeparatedStemClip { operation, .. }
            | Self::ArtifactVisualClip { operation, .. }
            | Self::AppliedTrackMatte { operation, .. }
            | Self::RetouchApply { operation, .. }
            | Self::TransformSamples { operation, .. }
            | Self::CropSamples { operation, .. }
            | Self::ColorPipeline { operation }
            | Self::ColorAdjustEffect { operation }
            | Self::MulticamSync { operation, .. } => operation,
        }
    }
}
