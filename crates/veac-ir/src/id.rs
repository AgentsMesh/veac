use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdError {
    expected_prefix: &'static str,
    value: String,
}

impl IdError {
    pub fn expected_prefix(&self) -> &'static str {
        self.expected_prefix
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for IdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid {} identifier {:?}",
            self.expected_prefix, self.value
        )
    }
}

impl std::error::Error for IdError {}

fn is_valid_id(value: &str, prefix: &str) -> bool {
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

fn digest_id(prefix: &str, digest: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(prefix.len() + digest.len() * 2);
    value.push_str(prefix);
    for byte in digest {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}

macro_rules! typed_id {
    ($name:ident, $prefix:literal, $pattern:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(#[schemars(regex(pattern = $pattern))] String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                if is_valid_id(&value, Self::PREFIX) {
                    Ok(Self(value))
                } else {
                    Err(IdError {
                        expected_prefix: Self::PREFIX,
                        value,
                    })
                }
            }

            /// Constructs a canonical typed ID from a fixed-width binary digest.
            pub fn from_digest(digest: [u8; 32]) -> Self {
                Self(digest_id(Self::PREFIX, digest))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            #[allow(dead_code)]
            pub(crate) fn is_valid(&self) -> bool {
                is_valid_id(&self.0, Self::PREFIX)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

typed_id!(ProjectId, "prj_", r"^prj_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(MaterialId, "med_", r"^med_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(SequenceId, "seq_", r"^seq_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(TrackId, "trk_", r"^trk_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(ItemId, "itm_", r"^itm_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(ApplyId, "apl_", r"^apl_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(
    ApplyStageId,
    "aps_",
    r"^aps_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(
    MulticamGroupId,
    "mcg_",
    r"^mcg_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(
    MulticamAngleId,
    "ang_",
    r"^ang_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(EffectId, "fx_", r"^fx_[A-Za-z0-9][A-Za-z0-9_-]{0,124}$");
typed_id!(
    AudioProcessorId,
    "aud_",
    r"^aud_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(EqBandId, "eqb_", r"^eqb_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(KeyframeId, "kf_", r"^kf_[A-Za-z0-9][A-Za-z0-9_-]{0,124}$");
typed_id!(
    RenderConfigId,
    "out_",
    r"^out_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(
    DeliverableId,
    "dlv_",
    r"^dlv_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(
    HlsRenditionId,
    "rnd_",
    r"^rnd_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(OperationId, "op_", r"^op_[A-Za-z0-9][A-Za-z0-9_-]{0,124}$");
typed_id!(
    AnnotationId,
    "ann_",
    r"^ann_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$"
);
typed_id!(RelationId, "rel_", r"^rel_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
typed_id!(BusId, "bus_", r"^bus_[A-Za-z0-9][A-Za-z0-9_-]{0,123}$");
