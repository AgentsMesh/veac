use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct DeriveArgs {
    pub input: PathBuf,
    pub spec: PathBuf,
    pub store: PathBuf,
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ProviderRunArgs {
    pub request: PathBuf,
    pub program: PathBuf,
    pub store: PathBuf,
    pub arguments: Vec<String>,
    pub response: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct ProviderProposeArgs {
    pub project: PathBuf,
    pub request: PathBuf,
    pub response: PathBuf,
    pub context: PathBuf,
    pub output: Option<PathBuf>,
}
