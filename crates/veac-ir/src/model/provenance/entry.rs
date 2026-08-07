use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AnnotationId, ApplyId, EntityAuthorship, MulticamGroupId, RelationId, RenderConfigId, TrackId,
};

macro_rules! authorship_entry {
    ($name:ident, $field:ident, $id:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            pub $field: $id,
            pub entity: EntityAuthorship,
        }
    };
}

authorship_entry!(TrackAuthorship, track_id, TrackId);
authorship_entry!(RelationAuthorship, relation_id, RelationId);
authorship_entry!(ApplyAuthorship, apply_id, ApplyId);
authorship_entry!(MulticamAuthorship, group_id, MulticamGroupId);
authorship_entry!(AnnotationAuthorship, annotation_id, AnnotationId);
authorship_entry!(DeliveryAuthorship, render_config_id, RenderConfigId);
