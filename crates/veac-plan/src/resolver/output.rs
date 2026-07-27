use super::PlanResolver;
use crate::{PlanOutputId, ResolvedOutput};

impl PlanResolver<'_> {
    pub(super) fn output(&self, id: PlanOutputId) -> ResolvedOutput {
        ResolvedOutput {
            id,
            render_config_id: self.config.id.clone(),
            sequence_id: self.config.sequence_id.clone(),
            width: self.config.width,
            height: self.config.height,
            frame_rate: self.config.frame_rate,
            deliverables: self.config.deliverables.clone(),
        }
    }
}
