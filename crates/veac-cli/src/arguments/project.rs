use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ProjectCommand {
    Check {
        project: PathBuf,
    },
    Inspect {
        project: PathBuf,
    },
    Graph {
        project: PathBuf,
    },
    Build {
        project: PathBuf,
        receipt: Option<PathBuf>,
    },
    Evidence {
        project: PathBuf,
        receipt: Option<PathBuf>,
    },
    Test {
        project: PathBuf,
        receipt: Option<PathBuf>,
    },
}
