use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::authoring::Document;

use super::diagnostic::{Diagnostic, Diagnostics};
use super::loader::{FileSystemLoader, LoadedSource, MemoryLoader, SourceLoader};
use super::provenance::ProvenanceMap;
use super::SourceIndex;
use super::{expand, parser, resolve};

#[derive(Debug, Clone)]
pub struct CompiledProgram {
    root_module: String,
    document: Document,
    expanded_source: String,
    provenance: ProvenanceMap,
    sources: BTreeMap<String, String>,
}

impl CompiledProgram {
    pub fn root_module(&self) -> &str {
        &self.root_module
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn expanded_source(&self) -> &str {
        &self.expanded_source
    }

    pub fn provenance(&self) -> &ProvenanceMap {
        &self.provenance
    }

    pub fn sources(&self) -> &BTreeMap<String, String> {
        &self.sources
    }

    pub fn source_index(&self) -> Result<SourceIndex, Diagnostics> {
        SourceIndex::build(&self.sources)
    }
}

pub fn compile_path(path: &Path) -> Result<CompiledProgram, Diagnostics> {
    compile_path_with_root(path).map(|(_, program)| program)
}

pub fn compile_path_with_root(path: &Path) -> Result<(PathBuf, CompiledProgram), Diagnostics> {
    let (loader, entry) = FileSystemLoader::for_entry(path).map_err(|message| {
        Diagnostics::one(Diagnostic::new(
            "PROGRAM_ENTRY_LOAD",
            path.display().to_string(),
            message,
            crate::authoring::Span::default(),
        ))
    })?;
    let root = loader.root().to_owned();
    compile_with_loader(entry, &loader).map(|program| (root, program))
}

pub fn compile_source(source: &str) -> Result<CompiledProgram, Diagnostics> {
    super::limits::check_source_size("main.veac", source, crate::authoring::Span::default())
        .map_err(Diagnostics::one)?;
    let entry = LoadedSource {
        id: "main.veac".to_owned(),
        source: source.to_owned(),
    };
    compile_with_loader(entry, &MemoryLoader::default())
}

/// Checks a standalone entry or module without resolving its imports.
pub fn check_source(path_label: &str, source: &str) -> Result<(), Diagnostics> {
    parser::parse(path_label, source)
        .map(|_| ())
        .map_err(Diagnostics)
}

pub fn compile_with_loader(
    entry: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<CompiledProgram, Diagnostics> {
    let root_path = entry.id.clone();
    let resolution = resolve::entry(entry, loader).map_err(Diagnostics)?;
    let (expanded_source, provenance) = expand::project(
        &resolution.entry,
        &resolution.scope,
        &resolution.component_catalog,
    )
    .map_err(Diagnostics::one)?;
    let document = crate::authoring::parse(&expanded_source).map_err(|errors| {
        Diagnostics(
            errors
                .as_slice()
                .iter()
                .map(|error| Diagnostic::new(error.code, &root_path, &error.message, error.span))
                .collect(),
        )
    })?;
    expand::validate_slot_media(
        &resolution.entry,
        &resolution.scope,
        &resolution.component_catalog,
        &document,
    )
    .map_err(Diagnostics::one)?;
    Ok(CompiledProgram {
        root_module: root_path,
        document,
        expanded_source,
        provenance,
        sources: resolution.sources,
    })
}
