use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ArtifactKind;
use veac_ir::{Rect, TimeRange};

use crate::validation::{normalized_rect, text};
use crate::{InputArtifact, MediaType, ProviderArtifact, ProviderResult, ReviewDecision, Validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemovalFill {
    Temporal,
    Spatial,
    Transparent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemovalTarget {
    pub id: String,
    pub label: String,
    pub range: TimeRange,
    pub initial_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemovalRequest {
    pub video: InputArtifact,
    pub targets: Vec<RemovalTarget>,
    pub fill: RemovalFill,
    pub correction_mattes: Vec<InputArtifact>,
}

impl Validate for RemovalRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video || self.targets.is_empty() {
            return crate::validation::invalid("removal requires video and targets");
        }
        let mut previous: Option<&str> = None;
        for target in &self.targets {
            text(&target.id, "removal target id")?;
            text(&target.label, "removal target label")?;
            crate::validation::range(target.range)?;
            normalized_rect(target.initial_rect)?;
            if previous.is_some_and(|value| value >= target.id.as_str()) {
                return crate::validation::invalid("removal targets must be unique and sorted");
            }
            previous = Some(&target.id);
        }
        for matte in &self.correction_mattes {
            matte.validate()?;
            if matte.media_type != MediaType::Matte {
                return crate::validation::invalid("removal corrections must be mattes");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemovalResult {
    pub video: ProviderArtifact,
    pub masks: Vec<ProviderArtifact>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for RemovalResult {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if !matches!(
            self.video.kind(),
            ArtifactKind::VideoMaster | ArtifactKind::ProxyVideo
        ) {
            return crate::validation::invalid("removal output must be a video artifact");
        }
        for mask in &self.masks {
            mask.validate()?;
            if mask.kind() != ArtifactKind::Matte {
                return crate::validation::invalid("removal masks must be matte artifacts");
            }
        }
        for value in &self.decisions {
            value.validate()?;
        }
        Ok(())
    }
}
