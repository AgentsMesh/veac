use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ProjectCommand {
    Check {
        project: PathBuf,
        package_roots: Vec<PathBuf>,
    },
    Inspect {
        project: PathBuf,
        package_roots: Vec<PathBuf>,
    },
    Graph {
        project: PathBuf,
        package_roots: Vec<PathBuf>,
    },
    Build {
        project: PathBuf,
        receipt: Option<PathBuf>,
        package_roots: Vec<PathBuf>,
    },
    Evidence {
        project: PathBuf,
        receipt: Option<PathBuf>,
        package_roots: Vec<PathBuf>,
    },
    Test {
        project: PathBuf,
        receipt: Option<PathBuf>,
        package_roots: Vec<PathBuf>,
    },
}
