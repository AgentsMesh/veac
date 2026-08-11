use veac_lang::program::expression::{ExactNumber, Value};

use super::{kind, unknown_variant, Decoder};
use crate::authored::EvidenceDecodeError;
use crate::{RangeExpectation, RationalTime};

impl<'a> Decoder<'a> {
    pub fn text(&self, value: &Value, path: &str) -> Result<String, EvidenceDecodeError> {
        match value {
            Value::Text(value) => Ok(value.to_string()),
            _ => Err(kind(path, "text", value)),
        }
    }

    pub fn identifier(&self, value: &Value, path: &str) -> Result<String, EvidenceDecodeError> {
        match value {
            Value::Identifier(value) => Ok(value.to_string()),
            _ => Err(kind(path, "identifier", value)),
        }
    }

    pub fn bool(&self, value: &Value, path: &str) -> Result<bool, EvidenceDecodeError> {
        match value {
            Value::Bool(value) => Ok(*value),
            _ => Err(kind(path, "bool", value)),
        }
    }

    pub fn integer(&self, value: &Value, path: &str) -> Result<i64, EvidenceDecodeError> {
        match value {
            Value::Integer(value) => Ok(*value),
            _ => Err(kind(path, "int", value)),
        }
    }

    pub fn u8(&self, value: &Value, path: &str) -> Result<u8, EvidenceDecodeError> {
        u8::try_from(self.integer(value, path)?)
            .map_err(|_| EvidenceDecodeError::new(path, "integer is outside u8 range"))
    }

    pub fn u32(&self, value: &Value, path: &str) -> Result<u32, EvidenceDecodeError> {
        u32::try_from(self.integer(value, path)?)
            .map_err(|_| EvidenceDecodeError::new(path, "integer is outside u32 range"))
    }

    pub fn u64(&self, value: &Value, path: &str) -> Result<u64, EvidenceDecodeError> {
        u64::try_from(self.integer(value, path)?)
            .map_err(|_| EvidenceDecodeError::new(path, "integer is outside u64 range"))
    }

    pub fn usize(&self, value: &Value, path: &str) -> Result<usize, EvidenceDecodeError> {
        usize::try_from(self.integer(value, path)?)
            .map_err(|_| EvidenceDecodeError::new(path, "integer is outside usize range"))
    }

    pub fn scalar(&self, value: &Value, path: &str) -> Result<f64, EvidenceDecodeError> {
        let Value::Scalar(value) = value else {
            return Err(kind(path, "scalar", value));
        };
        exact_f64(*value)
            .ok_or_else(|| EvidenceDecodeError::new(path, "scalar is outside exact f64 range"))
    }

    pub fn time(&self, value: &Value, path: &str) -> Result<RationalTime, EvidenceDecodeError> {
        let Value::Time(value) = value else {
            return Err(kind(path, "time", value));
        };
        let numerator = i64::try_from(value.numerator())
            .map_err(|_| EvidenceDecodeError::new(path, "time numerator is outside i64 range"))?;
        let denominator = u32::try_from(value.denominator())
            .map_err(|_| EvidenceDecodeError::new(path, "time timescale is outside u32 range"))?;
        RationalTime::new(numerator, denominator)
            .map_err(|error| EvidenceDecodeError::new(path, error.to_string()))
    }

    pub fn optional_identifier(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<Option<String>, EvidenceDecodeError> {
        let variant = self.variant(value, "OptionalIdentifier", path)?;
        match variant.name {
            "None" => Ok(None),
            "Some" => self
                .identifier(variant.fields.get("value")?, &variant.fields.path("value"))
                .map(Some),
            name => Err(unknown_variant(path, "OptionalIdentifier", name)),
        }
    }

    pub fn optional_scalar(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<Option<f64>, EvidenceDecodeError> {
        let variant = self.variant(value, "OptionalScalar", path)?;
        match variant.name {
            "None" => Ok(None),
            "Some" => self
                .scalar(variant.fields.get("value")?, &variant.fields.path("value"))
                .map(Some),
            name => Err(unknown_variant(path, "OptionalScalar", name)),
        }
    }

    pub fn optional_u32(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<Option<u32>, EvidenceDecodeError> {
        let variant = self.variant(value, "OptionalInteger", path)?;
        match variant.name {
            "None" => Ok(None),
            "Some" => self
                .u32(variant.fields.get("value")?, &variant.fields.path("value"))
                .map(Some),
            name => Err(unknown_variant(path, "OptionalInteger", name)),
        }
    }

    pub fn optional_range(
        &'a self,
        value: &'a Value,
        path: &str,
    ) -> Result<Option<RangeExpectation>, EvidenceDecodeError> {
        let variant = self.variant(value, "OptionalRangeExpectation", path)?;
        match variant.name {
            "None" => Ok(None),
            "Some" => super::super::expectation::range(
                self,
                variant.fields.get("value")?,
                &variant.fields.path("value"),
            )
            .map(Some),
            name => Err(unknown_variant(path, "OptionalRangeExpectation", name)),
        }
    }
}

fn exact_f64(value: ExactNumber) -> Option<f64> {
    const MAX_SAFE_INTEGER: u128 = 9_007_199_254_740_991;
    if value.numerator().unsigned_abs() > MAX_SAFE_INTEGER
        || value.denominator() as u128 > MAX_SAFE_INTEGER
    {
        return None;
    }
    let result = value.numerator() as f64 / value.denominator() as f64;
    result.is_finite().then_some(result)
}
