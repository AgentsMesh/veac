use crate::program::expression::{UnitDimension, UnitSuffix};

use super::catalog::{identifier_policy, Catalog};
use super::{CanonicalRole, GrammarPosition, VocabularyCategory};

pub(super) fn add(catalog: &mut Catalog) {
    for unit in UnitSuffix::ALL {
        add_at(catalog, unit, core_position(unit.dimension()));
        if unit.is_executable_expression() {
            add_at(catalog, unit, GrammarPosition::ExpressionNumericUnit);
        }
    }
    add_at(
        catalog,
        UnitSuffix::Repetitions,
        GrammarPosition::GifPlaybackValuePosition,
    );
}

fn add_at(catalog: &mut Catalog, unit: UnitSuffix, position: GrammarPosition) {
    catalog.add(
        unit.as_str(),
        identifier_policy(unit.as_str()),
        VocabularyCategory::UnitSuffix,
        position,
        CanonicalRole::NumericUnit,
    );
}

const fn core_position(dimension: UnitDimension) -> GrammarPosition {
    match dimension {
        UnitDimension::Time => GrammarPosition::CoreTimeUnit,
        UnitDimension::Length => GrammarPosition::CoreLengthUnit,
        UnitDimension::Percent => GrammarPosition::CorePercentUnit,
        UnitDimension::Angle => GrammarPosition::CoreAngleUnit,
        UnitDimension::FrameRate => GrammarPosition::CoreFrameRateUnit,
        UnitDimension::Frequency => GrammarPosition::CoreFrequencyUnit,
        UnitDimension::Decibels => GrammarPosition::CoreDecibelUnit,
        UnitDimension::TruePeak => GrammarPosition::CoreTruePeakUnit,
        UnitDimension::Loudness => GrammarPosition::CoreLoudnessUnit,
        UnitDimension::Temperature => GrammarPosition::CoreTemperatureUnit,
        UnitDimension::Exposure => GrammarPosition::CoreExposureUnit,
        UnitDimension::BitRate => GrammarPosition::CoreBitRateUnit,
        UnitDimension::BitCount => GrammarPosition::CoreBitCountUnit,
        UnitDimension::Repetition => GrammarPosition::CoreRepetitionUnit,
    }
}
