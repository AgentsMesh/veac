use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{MaterialId, MulticamAngleId, MulticamSyncBasis, RationalTime};

use crate::validation::{invalid, probability, time};
use crate::{InputArtifact, MediaType, ProviderResult, ReviewDecision, Validate};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSyncInput {
    pub angle_id: MulticamAngleId,
    pub material_id: MaterialId,
    pub media: InputArtifact,
    pub timecode_start: Option<RationalTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSyncRequest {
    pub basis: MulticamSyncBasis,
    pub reference_angle_id: MulticamAngleId,
    pub angles: Vec<MulticamSyncInput>,
    pub maximum_offset: RationalTime,
}

impl Validate for MulticamSyncRequest {
    fn validate(&self) -> ProviderResult<()> {
        if self.basis == MulticamSyncBasis::Manual {
            return invalid("automatic multicam sync cannot use the manual basis");
        }
        time(self.maximum_offset)?;
        if self.maximum_offset.value <= 0 || self.angles.len() < 2 {
            return invalid("multicam sync requires two angles and a positive offset limit");
        }
        let mut previous = None;
        let mut has_reference = false;
        for angle in &self.angles {
            if MulticamAngleId::new(angle.angle_id.as_str()).is_err()
                || MaterialId::new(angle.material_id.as_str()).is_err()
            {
                return invalid("multicam sync input IDs are invalid");
            }
            if previous.is_some_and(|id: &str| id >= angle.angle_id.as_str()) {
                return invalid("multicam sync inputs must be unique and sorted");
            }
            angle.media.validate()?;
            validate_input(self.basis, angle)?;
            has_reference |= angle.angle_id == self.reference_angle_id;
            previous = Some(angle.angle_id.as_str());
        }
        if !has_reference {
            return invalid("multicam reference angle is not present in the inputs");
        }
        Ok(())
    }
}

fn validate_input(basis: MulticamSyncBasis, angle: &MulticamSyncInput) -> ProviderResult<()> {
    match basis {
        MulticamSyncBasis::Audio
            if angle.media.media_type == MediaType::Audio && angle.timecode_start.is_none() =>
        {
            Ok(())
        }
        MulticamSyncBasis::Timecode
            if angle.media.media_type == MediaType::Video && angle.timecode_start.is_some() =>
        {
            time(angle.timecode_start.expect("checked above"))
        }
        _ => invalid("multicam sync inputs do not match the selected basis"),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSyncOffset {
    pub angle_id: MulticamAngleId,
    pub material_id: MaterialId,
    pub source_offset: RationalTime,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MulticamSyncResult {
    pub basis: MulticamSyncBasis,
    pub reference_angle_id: MulticamAngleId,
    pub offsets: Vec<MulticamSyncOffset>,
    pub decisions: Vec<ReviewDecision>,
}

impl Validate for MulticamSyncResult {
    fn validate(&self) -> ProviderResult<()> {
        if self.basis == MulticamSyncBasis::Manual || self.offsets.len() < 2 {
            return invalid("automatic multicam sync result is incomplete");
        }
        let mut previous = None;
        let mut has_reference = false;
        for value in &self.offsets {
            if MulticamAngleId::new(value.angle_id.as_str()).is_err()
                || MaterialId::new(value.material_id.as_str()).is_err()
            {
                return invalid("multicam sync result IDs are invalid");
            }
            if previous.is_some_and(|id: &str| id >= value.angle_id.as_str()) {
                return invalid("multicam sync offsets must be unique and sorted");
            }
            time(value.source_offset)?;
            if value.source_offset.value < 0 {
                return invalid("multicam source offsets cannot be negative");
            }
            probability(value.confidence, "multicam sync confidence")?;
            has_reference |= value.angle_id == self.reference_angle_id;
            previous = Some(value.angle_id.as_str());
        }
        for value in &self.decisions {
            value.validate()?;
        }
        if !has_reference {
            return invalid("multicam result reference angle is absent");
        }
        Ok(())
    }
}
