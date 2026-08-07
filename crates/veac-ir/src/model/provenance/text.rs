use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! typed_text {
    ($name:ident, $pattern:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(#[schemars(regex(pattern = $pattern))] String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) fn is_valid(&self, maximum: usize) -> bool {
                !self.0.is_empty()
                    && self.0.len() <= maximum
                    && !self.0.chars().any(char::is_control)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

typed_text!(LogicalPathSegment, r"^.{1,128}$");
typed_text!(AuthoredDefinitionId, r"^.{1,256}$");
typed_text!(AuthoredFunctionName, r"^.{1,256}$");
typed_text!(AuthoredSourceId, r"^.{1,1024}$");
typed_text!(LoopLogicalKey, r"^loop_[0-9a-f]{64}$");
typed_text!(Sha256Digest, r"^[0-9a-f]{64}$");

impl LogicalPathSegment {
    pub(crate) fn is_valid_logical_key(&self) -> bool {
        let value = self.as_str();
        self.is_valid(128)
            && value
                .chars()
                .next()
                .is_some_and(|value| value.is_ascii_alphabetic() || value == '_')
            && value
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
            && !value.ends_with('-')
            && !value.contains("--")
            && !matches!(value, "true" | "false")
    }
}
