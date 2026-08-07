#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitDimension {
    Time,
    Length,
    Percent,
    Angle,
    FrameRate,
    Frequency,
    Decibels,
    TruePeak,
    Loudness,
    Temperature,
    Exposure,
    BitRate,
    BitCount,
    Repetition,
}

crate::define_syntax_tokens! {
    array
    pub enum UnitSuffix {
        Seconds => "s",
        Milliseconds => "ms",
        Microseconds => "us",
        Pixels => "px",
        Percent => "%",
        Degrees => "deg",
        FramesPerSecond => "fps",
        Hertz => "hz",
        Kilohertz => "khz",
        Decibels => "db",
        DecibelsTruePeak => "dbtp",
        LoudnessUnitsFullScale => "lufs",
        LoudnessUnits => "lu",
        Kelvin => "k",
        ExposureStops => "stops",
        BitsPerSecond => "bps",
        KilobitsPerSecond => "kbps",
        MegabitsPerSecond => "mbps",
        Bits => "bit",
        Kilobits => "kbit",
        Megabits => "mbit",
        Repetitions => "times",
    }
}

impl UnitSuffix {
    pub const EXECUTABLE_EXPRESSION: [Self; 6] = [
        Self::Seconds,
        Self::Milliseconds,
        Self::Microseconds,
        Self::Pixels,
        Self::Percent,
        Self::Degrees,
    ];
    pub const TIME: [Self; 3] = [Self::Seconds, Self::Milliseconds, Self::Microseconds];
    pub const SAMPLE_RATE: [Self; 2] = [Self::Hertz, Self::Kilohertz];
    pub const BIT_RATE: [Self; 3] = [
        Self::BitsPerSecond,
        Self::KilobitsPerSecond,
        Self::MegabitsPerSecond,
    ];
    pub const BIT_COUNT: [Self; 3] = [Self::Bits, Self::Kilobits, Self::Megabits];

    pub fn split_literal(raw: &str) -> (&str, &str) {
        let boundary = raw
            .char_indices()
            .find(|(_, value)| value.is_ascii_alphabetic() || *value == '%')
            .map_or(raw.len(), |(index, _)| index);
        raw.split_at(boundary)
    }

    pub const fn scale(self) -> (u64, u64) {
        match self {
            Self::Milliseconds => (1, 1_000),
            Self::Microseconds => (1, 1_000_000),
            Self::Kilohertz | Self::KilobitsPerSecond | Self::Kilobits => (1_000, 1),
            Self::MegabitsPerSecond | Self::Megabits => (1_000_000, 1),
            _ => (1, 1),
        }
    }

    pub const fn dimension(self) -> UnitDimension {
        match self {
            Self::Seconds | Self::Milliseconds | Self::Microseconds => UnitDimension::Time,
            Self::Pixels => UnitDimension::Length,
            Self::Percent => UnitDimension::Percent,
            Self::Degrees => UnitDimension::Angle,
            Self::FramesPerSecond => UnitDimension::FrameRate,
            Self::Hertz | Self::Kilohertz => UnitDimension::Frequency,
            Self::Decibels => UnitDimension::Decibels,
            Self::DecibelsTruePeak => UnitDimension::TruePeak,
            Self::LoudnessUnitsFullScale | Self::LoudnessUnits => UnitDimension::Loudness,
            Self::Kelvin => UnitDimension::Temperature,
            Self::ExposureStops => UnitDimension::Exposure,
            Self::BitsPerSecond | Self::KilobitsPerSecond | Self::MegabitsPerSecond => {
                UnitDimension::BitRate
            }
            Self::Bits | Self::Kilobits | Self::Megabits => UnitDimension::BitCount,
            Self::Repetitions => UnitDimension::Repetition,
        }
    }

    pub const fn is_executable_expression(self) -> bool {
        matches!(
            self,
            Self::Seconds
                | Self::Milliseconds
                | Self::Microseconds
                | Self::Pixels
                | Self::Percent
                | Self::Degrees
        )
    }
}
