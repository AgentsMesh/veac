mod outputs;
mod structure;
mod timeline;
mod visual;

use crate::Project;

use super::Validator;

impl Validator {
    pub(super) fn render_budget(&mut self, project: &Project) {
        structure::validate(self, project);
        timeline::validate(self, project);
        outputs::validate(self, project);
        visual::validate(self, project);
    }
}
