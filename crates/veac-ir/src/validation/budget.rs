mod outputs;
mod structure;
mod timeline;
mod visual;

use crate::{Project, TemporalProgramLibrary};

use super::Validator;

impl Validator {
    pub(super) fn structural_render_budget(
        &mut self,
        project: &Project,
        temporal: &TemporalProgramLibrary,
    ) {
        structure::validate(self, project, temporal);
        timeline::validate(self, project);
        outputs::validate(self, project);
    }

    pub(super) fn visual_render_budget(&mut self, project: &Project) {
        visual::validate(self, project);
    }
}
