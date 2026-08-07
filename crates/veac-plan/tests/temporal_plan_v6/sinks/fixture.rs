use veac_plan::canonical::*;

use crate::support;

use super::{content, library};

pub fn project() -> (ProjectEnvelope, Vec<TemporalBindingId>) {
    let mut project = support::project();
    project.project.sequences[0].settings.width = 64;
    project.project.sequences[0].settings.height = 64;
    let raster = project.project.render_configs[0].raster.as_mut().unwrap();
    raster.width = 64;
    raster.height = 64;
    let ids = library::install(&mut project);
    let template = project.project.sequences[0].tracks[0].clips[0].clone();
    content::bind_media_clip(&mut project.project.sequences[0].tracks[0].clips[0], &ids);
    content::add_text_clip(&mut project, template, &ids);
    content::add_apply(&mut project, &ids);
    (project, ids.values())
}
