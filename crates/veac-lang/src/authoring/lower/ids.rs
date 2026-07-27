use crate::authoring::Identifier;
use veac_ir::{
    AnnotationId, ApplyId, ApplyStageId, BusId, DeliverableId, EffectId, ItemId, KeyframeId,
    MaterialId, MulticamAngleId, MulticamGroupId, ProjectId, RelationId, RenderConfigId,
    SequenceId, TrackId,
};

use super::context::Context;

macro_rules! id_fn {
    ($name:ident, $ty:ty, $prefix:literal) => {
        pub fn $name(ctx: &mut Context, value: &Identifier) -> Option<$ty> {
            let raw = format!("{}{}", $prefix, value.value);
            match <$ty>::new(raw) {
                Ok(id) => Some(id),
                Err(error) => {
                    ctx.error("AUTHORING_LOWER_ID", format!("{error:?}"), value.span);
                    None
                }
            }
        }
    };
}

id_fn!(project, ProjectId, "prj_");
id_fn!(material, MaterialId, "med_");
id_fn!(sequence, SequenceId, "seq_");
id_fn!(track, TrackId, "trk_");
id_fn!(item, ItemId, "itm_");
id_fn!(apply, ApplyId, "apl_");
id_fn!(apply_stage, ApplyStageId, "aps_");
id_fn!(effect, EffectId, "fx_");
id_fn!(deliverable, DeliverableId, "dlv_");
id_fn!(render_config, RenderConfigId, "out_");

id_fn!(key, KeyframeId, "kf_");

id_fn!(bus_id, BusId, "bus_");
id_fn!(relation, RelationId, "rel_");
id_fn!(annotation, AnnotationId, "ann_");
id_fn!(multicam, MulticamGroupId, "mcg_");
id_fn!(angle, MulticamAngleId, "ang_");
