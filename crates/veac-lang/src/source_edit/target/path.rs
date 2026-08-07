use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::SourceTemporalProperty;

mod identity;
pub(super) mod kind;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceNodePath {
    Input {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        input: String,
    },
    Constant {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        constant: String,
    },
    Function {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        function: String,
    },
    Method {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        receiver: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        method: String,
    },
    Implementation {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        receiver: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        implementation: String,
    },
    Struct {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        structure: String,
    },
    StructField {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        structure: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        field: String,
    },
    Enum {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        enumeration: String,
    },
    EnumVariant {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        enumeration: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        variant: String,
    },
    EnumVariantField {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        enumeration: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        variant: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        field: String,
    },
    Temporal {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
        property: SourceTemporalProperty,
    },
    TemporalClipMask {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
        mask_index: u32,
        property: SourceTemporalProperty,
    },
    TemporalClipEffect {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        effect: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        parameter: String,
        property: SourceTemporalProperty,
    },
    TemporalApply {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
        property: SourceTemporalProperty,
    },
    TemporalApplyMask {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
        mask_index: u32,
        property: SourceTemporalProperty,
    },
    TemporalApplyEffect {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        apply: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        stage: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        effect: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        parameter: String,
        property: SourceTemporalProperty,
    },
    Item {
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        project: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        sequence: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        layer: String,
        #[schemars(with = "super::schema::CanonicalNameSchema")]
        item: String,
    },
}
