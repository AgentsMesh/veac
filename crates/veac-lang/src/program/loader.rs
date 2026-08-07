mod filesystem;
mod memory;
mod path;

pub use filesystem::FileSystemLoader;
pub(crate) use memory::MemoryLoader;
pub(crate) use path::validate_source_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSource {
    pub id: String,
    pub source: String,
}

/// Resolves imports to stable, root-relative UTF-8 source IDs.
pub trait SourceLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String>;
}
