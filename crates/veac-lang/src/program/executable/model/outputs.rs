use super::BuiltProgram;

impl BuiltProgram {
    pub fn deliverable_by_logical_key(
        &self,
        config_id: &veac_ir::RenderConfigId,
        key: &str,
    ) -> Option<&veac_ir::Deliverable> {
        let authorship = self.envelope.project.authorship.as_ref()?;
        let delivery = authorship
            .deliveries
            .iter()
            .find(|delivery| delivery.render_config_id == *config_id)?;
        let mut path = delivery
            .entity
            .logical_path
            .iter()
            .map(|segment| segment.as_str())
            .collect::<Vec<_>>();
        let kind = path.len().checked_sub(2)?;
        if path.get(kind).copied() != Some("delivery") {
            return None;
        }
        path.remove(kind);
        path.push(key);
        let id = super::super::lower::id::deliverable(&path);
        self.envelope
            .project
            .render_configs
            .iter()
            .find(|config| config.id == *config_id)?
            .deliverables
            .iter()
            .find(|deliverable| deliverable.id == id)
    }
}

#[cfg(test)]
#[path = "outputs/tests.rs"]
mod tests;
