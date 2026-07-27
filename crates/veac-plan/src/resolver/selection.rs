use veac_ir::{ProjectEnvelope, RenderConfig, RenderConfigId};

use crate::{ResolutionDiagnostic, ResolutionErrorKind, ResolutionErrors};

pub(super) fn select_configs<'a>(
    envelope: &'a ProjectEnvelope,
    selected: Option<&RenderConfigId>,
) -> Result<Vec<&'a RenderConfig>, ResolutionErrors> {
    let mut configs: Vec<_> = match selected {
        Some(id) => envelope
            .project
            .render_configs
            .iter()
            .filter(|config| config.id == *id)
            .collect(),
        None => envelope.project.render_configs.iter().collect(),
    };
    configs.sort_by(|left, right| left.id.cmp(&right.id));
    if let (Some(id), true) = (selected, configs.is_empty()) {
        Err(ResolutionErrors::new(vec![ResolutionDiagnostic::new(
            ResolutionErrorKind::RenderConfigNotFound,
            "RENDER_CONFIG_NOT_FOUND",
            Some(id.to_string()),
            "/project/render_configs",
            format!("render config {id:?} does not exist"),
        )]))
    } else {
        Ok(configs)
    }
}
