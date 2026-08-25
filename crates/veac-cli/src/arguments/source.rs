use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct BuildSourceArgs {
    pub(crate) source: PathBuf,
    pub(crate) emit_ir: Option<PathBuf>,
    pub(crate) inputs: Option<PathBuf>,
    pub(crate) inline_inputs: Vec<String>,
    pub(crate) material_root: Option<PathBuf>,
    pub(crate) package_roots: Vec<PathBuf>,
    pub(crate) revision: u64,
}

#[derive(Debug)]
pub(crate) struct CheckSourceArgs {
    pub(crate) source: PathBuf,
    pub(crate) inputs: Option<PathBuf>,
    pub(crate) inline_inputs: Vec<String>,
    pub(crate) package_roots: Vec<PathBuf>,
    pub(crate) revision: u64,
}

#[derive(Debug)]
pub(crate) struct FormatSourceArgs {
    pub(crate) source: PathBuf,
    pub(crate) check: bool,
    pub(crate) stdout: bool,
    pub(crate) package_roots: Vec<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct SourceGraphArgs {
    pub(crate) source: PathBuf,
    pub(crate) package_roots: Vec<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct SourceEditArgs {
    pub(crate) source: PathBuf,
    pub(crate) source_edit_batch: PathBuf,
    pub(crate) inputs: Option<PathBuf>,
    pub(crate) inline_inputs: Vec<String>,
    pub(crate) package_roots: Vec<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) dry_run: bool,
}
