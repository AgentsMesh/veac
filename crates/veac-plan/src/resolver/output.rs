use super::PlanResolver;
use crate::{PlanOutputId, ResolvedOutput};

impl PlanResolver<'_> {
    pub(super) fn output(&self, id: PlanOutputId) -> ResolvedOutput {
        ResolvedOutput {
            id,
            render_config_id: self.config.id.clone(),
            sequence_id: self.config.sequence_id.clone(),
            raster: self.config.raster.clone(),
            deliverables: self.config.deliverables.clone(),
        }
    }

    pub(super) fn output_id(&mut self) -> Option<PlanOutputId> {
        let suffix = self.config.id.as_str().strip_prefix("out_")?;
        match PlanOutputId::new(format!("pout_{suffix}")) {
            Ok(id) => Some(id),
            Err(error) => {
                self.push_internal(
                    "PLAN_OUTPUT_ID",
                    self.config.id.to_string(),
                    error.to_string(),
                );
                None
            }
        }
    }
}
