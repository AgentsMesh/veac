use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanIdError {
    expected_prefix: &'static str,
    value: String,
}

impl fmt::Display for PlanIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid plan identifier {:?}; expected prefix {:?}",
            self.value, self.expected_prefix
        )
    }
}

impl std::error::Error for PlanIdError {}

fn valid_id(value: &str, prefix: &str, maximum_len: usize) -> bool {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return false;
    };
    !suffix.is_empty()
        && value.len() <= maximum_len
        && suffix
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

macro_rules! plan_id {
    ($name:ident, $prefix:literal, $pattern:literal, $maximum_len:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(#[schemars(regex(pattern = $pattern))] String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn new(value: impl Into<String>) -> Result<Self, PlanIdError> {
                let value = value.into();
                valid_id(&value, Self::PREFIX, $maximum_len)
                    .then_some(Self(value.clone()))
                    .ok_or(PlanIdError {
                        expected_prefix: Self::PREFIX,
                        value,
                    })
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

plan_id!(
    PlanInputId,
    "pin_",
    r"^pin_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$",
    128
);
plan_id!(
    PlanOutputId,
    "pout_",
    r"^pout_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$",
    129
);
