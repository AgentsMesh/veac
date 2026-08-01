use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    Deliverable, DeliverableId, DeliverableKind, RasterSettings, RenderConfigId, SequenceId,
};

use super::PlanOutputId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedOutput {
    pub id: PlanOutputId,
    pub render_config_id: RenderConfigId,
    pub sequence_id: SequenceId,
    pub raster: Option<RasterSettings>,
    pub deliverables: Vec<Deliverable>,
}

impl ResolvedOutput {
    pub fn deliverable(&self, id: &DeliverableId) -> Option<&Deliverable> {
        self.deliverables.iter().find(|value| value.id == *id)
    }

    pub fn video_deliverable(
        &self,
        id: &DeliverableId,
    ) -> Option<(&Deliverable, &veac_ir::VideoDeliverable)> {
        let deliverable = self.deliverable(id)?;
        let DeliverableKind::Video(settings) = &deliverable.kind else {
            return None;
        };
        Some((deliverable, settings))
    }

    pub fn video_deliverable_mut(
        &mut self,
        id: &DeliverableId,
    ) -> Option<&mut veac_ir::VideoDeliverable> {
        self.deliverables
            .iter_mut()
            .find(|deliverable| deliverable.id == *id)
            .and_then(|deliverable| match &mut deliverable.kind {
                DeliverableKind::Video(settings) => Some(settings),
                _ => None,
            })
    }
}
