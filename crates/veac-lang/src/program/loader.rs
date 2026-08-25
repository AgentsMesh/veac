mod composite;
mod filesystem;
mod memory;
mod path;

pub use composite::CompositeSourceLoader;
pub use filesystem::FileSystemLoader;
pub(crate) use memory::MemoryLoader;
pub(crate) use path::validate_source_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSource {
    pub id: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceAuthority {
    Project,
    ReadOnlyDependency,
}

/// Resolves imports to stable canonical UTF-8 source IDs.
pub trait SourceLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String>;
    /// Declares who may mutate a canonical source ID returned by this loader.
    ///
    /// The result must be deterministic and immutable for one loader snapshot.
    /// Prepared graphs capture it as provenance; later loaders cannot reauthorize it.
    fn authority(&self, source_id: &str) -> SourceAuthority;
}
