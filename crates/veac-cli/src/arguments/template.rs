use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum TemplateCommand {
    /// Produce an atomic EditBatch that fills every typed slot in one project.
    Propose {
        project: PathBuf,
        request: PathBuf,
        output: Option<PathBuf>,
    },
}
