use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ArtifactKind;
use veac_ir::Rect;

use crate::validation::{normalized_rect, text};
use crate::{
    InputArtifact, MediaType, ProviderArtifact, ProviderResult, ReviewDecision, ScalarSample,
    Validate,
};

mod removal;
pub use removal::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RetouchKind {
    Face,
    Skin,
    Body,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchTarget {
    pub id: String,
    pub kind: RetouchKind,
    pub initial_rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParameterCurve {
    pub parameter: String,
    pub samples: Vec<ScalarSample>,
}

impl Validate for ParameterCurve {
    fn validate(&self) -> ProviderResult<()> {
        text(&self.parameter, "parameter name")?;
        for sample in &self.samples {
            sample.validate()?;
        }
        crate::validation::increasing(
            &self
                .samples
                .iter()
                .map(|sample| sample.time)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchRequest {
    pub video: InputArtifact,
    pub targets: Vec<RetouchTarget>,
    pub controls: Vec<ParameterCurve>,
    pub correction_mattes: Vec<InputArtifact>,
}

impl Validate for RetouchRequest {
    fn validate(&self) -> ProviderResult<()> {
        self.video.validate()?;
        if self.video.media_type != MediaType::Video || self.targets.is_empty() {
            return crate::validation::invalid("retouch requires video and targets");
        }
        let mut previous: Option<&str> = None;
        for target in &self.targets {
            text(&target.id, "retouch target id")?;
            normalized_rect(target.initial_rect)?;
            if previous.is_some_and(|value| value >= target.id.as_str()) {
                return crate::validation::invalid("retouch targets must be unique and sorted");
            }
            previous = Some(&target.id);
        }
        validate_curves(&self.controls)?;
        for matte in &self.correction_mattes {
            matte.validate()?;
            if matte.media_type != MediaType::Matte {
                return crate::validation::invalid("retouch corrections must be mattes");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchResult {
    pub masks: Vec<ProviderArtifact>,
    pub controls: Vec<ParameterCurve>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for RetouchResult {
    fn validate(&self) -> ProviderResult<()> {
        for mask in &self.masks {
            mask.validate()?;
            if mask.kind() != ArtifactKind::Matte {
                return crate::validation::invalid("retouch masks must be matte artifacts");
            }
        }
        validate_curves(&self.controls)?;
        for value in &self.decisions {
            value.validate()?;
        }
        Ok(())
    }
}

fn validate_curves(values: &[ParameterCurve]) -> ProviderResult<()> {
    if values.is_empty() {
        return crate::validation::invalid("retouch controls cannot be empty");
    }
    for value in values {
        value.validate()?;
    }
    if values
        .windows(2)
        .all(|pair| pair[0].parameter < pair[1].parameter)
    {
        Ok(())
    } else {
        crate::validation::invalid("parameter curves must be unique and sorted")
    }
}
