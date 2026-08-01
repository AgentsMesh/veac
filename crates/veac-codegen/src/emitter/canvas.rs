use veac_plan::canonical::Rational;
use veac_plan::{ResolvedOutput, ResolvedSequence};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Canvas {
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
}

impl Canvas {
    pub fn from_output(output: &ResolvedOutput) -> Option<Self> {
        output.raster.as_ref().map(|raster| Self {
            width: raster.width,
            height: raster.height,
            frame_rate: raster.frame_rate,
        })
    }

    pub fn from_sequence(sequence: &ResolvedSequence) -> Self {
        Self {
            width: sequence.settings.width,
            height: sequence.settings.height,
            frame_rate: sequence.settings.frame_rate,
        }
    }
}
