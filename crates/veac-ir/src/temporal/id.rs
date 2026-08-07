use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalIdError {
    expected_prefix: &'static str,
    value: String,
}

impl TemporalIdError {
    pub fn expected_prefix(&self) -> &'static str {
        self.expected_prefix
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for TemporalIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid {} identifier {:?}",
            self.expected_prefix, self.value
        )
    }
}

impl std::error::Error for TemporalIdError {}

fn valid(value: &str, prefix: &str) -> bool {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return false;
    };
    !suffix.is_empty()
        && value.len() <= 128
        && suffix
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

macro_rules! temporal_id {
    ($name:ident, $prefix:literal, $pattern:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(#[schemars(regex(pattern = $pattern))] String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn new(value: impl Into<String>) -> Result<Self, TemporalIdError> {
                let value = value.into();
                valid(&value, Self::PREFIX)
                    .then_some(Self(value.clone()))
                    .ok_or(TemporalIdError {
                        expected_prefix: Self::PREFIX,
                        value,
                    })
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) fn is_valid(&self) -> bool {
                valid(&self.0, Self::PREFIX)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

temporal_id!(
    TemporalProgramId,
    "tpg_",
    r"^tpg_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
temporal_id!(
    TemporalBindingId,
    "tbd_",
    r"^tbd_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
temporal_id!(
    TemporalParameterId,
    "tpm_",
    r"^tpm_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
temporal_id!(
    TemporalProvenanceId,
    "tpv_",
    r"^tpv_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
temporal_id!(
    TemporalSourceId,
    "src_",
    r"^src_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
temporal_id!(
    TemporalDefinitionId,
    "def_",
    r"^def_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
temporal_id!(
    TemporalLogicalKey,
    "key_",
    r"^key_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);

macro_rules! numeric_id {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

numeric_id!(TemporalNodeId);
numeric_id!(TemporalInputId);
