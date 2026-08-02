use std::{cmp::Ordering, fmt};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::MAX_SAFE_INTEGER;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    ZeroTimescale,
    MismatchedTimescale,
    NonPositiveDuration,
    UnsafeInteger,
    Overflow,
}

impl fmt::Display for TimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroTimescale => "timescale must be greater than zero",
            Self::MismatchedTimescale => "time values use different timescales",
            Self::NonPositiveDuration => "duration must be greater than zero",
            Self::UnsafeInteger => "integer is outside the exact I-JSON range",
            Self::Overflow => "time arithmetic overflowed",
        })
    }
}

impl std::error::Error for TimeError {}

/// An exact time value. Project timeline values normally share the project's timescale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RationalTime {
    #[schemars(range(min = -9007199254740991i64, max = 9007199254740991i64))]
    pub value: i64,
    pub timescale: u32,
}

impl RationalTime {
    pub fn new(value: i64, timescale: u32) -> Result<Self, TimeError> {
        if timescale == 0 {
            Err(TimeError::ZeroTimescale)
        } else if !safe_i64(value) {
            Err(TimeError::UnsafeInteger)
        } else {
            Ok(Self { value, timescale })
        }
    }

    pub fn zero(timescale: u32) -> Result<Self, TimeError> {
        Self::new(0, timescale)
    }

    pub fn checked_add(self, other: Self) -> Result<Self, TimeError> {
        if self.timescale != other.timescale {
            return Err(TimeError::MismatchedTimescale);
        }
        let value = self
            .value
            .checked_add(other.value)
            .ok_or(TimeError::Overflow)?;
        if !safe_i64(value) {
            return Err(TimeError::UnsafeInteger);
        }
        Ok(Self {
            value,
            timescale: self.timescale,
        })
    }

    pub fn is_valid(self) -> bool {
        self.timescale > 0 && safe_i64(self.value)
    }
}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !self.is_valid() || !other.is_valid() {
            return None;
        }
        let left = i128::from(self.value) * i128::from(other.timescale);
        let right = i128::from(other.value) * i128::from(self.timescale);
        Some(left.cmp(&right))
    }
}

/// A half-open interval `[start, start + duration)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimeRange {
    pub start: RationalTime,
    pub duration: RationalTime,
}

impl TimeRange {
    pub fn new(start: RationalTime, duration: RationalTime) -> Result<Self, TimeError> {
        if start.timescale != duration.timescale {
            Err(TimeError::MismatchedTimescale)
        } else if duration.value <= 0 {
            Err(TimeError::NonPositiveDuration)
        } else {
            Ok(Self { start, duration })
        }
    }

    pub fn end(self) -> Result<RationalTime, TimeError> {
        self.start.checked_add(self.duration)
    }
}

/// An exact ratio used for frame rates and constant playback rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rational {
    #[schemars(range(min = -9007199254740991i64, max = 9007199254740991i64))]
    pub numerator: i64,
    pub denominator: u32,
}

impl Rational {
    pub fn new(numerator: i64, denominator: u32) -> Result<Self, TimeError> {
        if denominator == 0 {
            Err(TimeError::ZeroTimescale)
        } else if !safe_i64(numerator) {
            Err(TimeError::UnsafeInteger)
        } else {
            let divisor = gcd(numerator.unsigned_abs(), u64::from(denominator));
            Ok(Self {
                numerator: numerator / i64::try_from(divisor).expect("divisor fits in i64"),
                denominator: denominator / u32::try_from(divisor).expect("divisor fits in u32"),
            })
        }
    }

    pub fn is_positive(self) -> bool {
        self.numerator > 0 && rational_is_canonical(self)
    }
}

pub(crate) fn safe_i64(value: i64) -> bool {
    value.unsigned_abs() <= MAX_SAFE_INTEGER
}

pub(crate) fn safe_u64(value: u64) -> bool {
    value <= MAX_SAFE_INTEGER
}

pub(crate) fn rational_is_canonical(value: Rational) -> bool {
    value.denominator > 0
        && safe_i64(value.numerator)
        && gcd(value.numerator.unsigned_abs(), u64::from(value.denominator)) == 1
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
