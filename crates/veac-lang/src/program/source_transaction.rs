use std::fmt;
use std::path::{Path, PathBuf};

use crate::source_edit::{
    apply_borrowed_text_edits, source_graph_revision, validate_source_edit_batch, BorrowedTextEdit,
    SourceEditBatch, SourceEditError, SourceModule, SourceRevision,
};

use super::compile::{self, CompiledProgram};
use super::loader::{LoadedSource, MemoryLoader};
use super::{compile_path_with_root, Diagnostics};

#[derive(Debug)]
pub enum SourceTransactionError {
    Program(Diagnostics),
    Contract(SourceEditError),
    TargetNotFound { operation: usize },
    MultipleModules,
    Lowering(crate::authoring::Diagnostics),
}

impl fmt::Display for SourceTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(error) => write!(formatter, "{error}"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::TargetNotFound { operation } => {
                write!(
                    formatter,
                    "source-edit operation {operation} has no authored expression site"
                )
            }
            Self::MultipleModules => {
                formatter.write_str("one source-edit batch may modify only one module")
            }
            Self::Lowering(error) => write!(formatter, "edited source does not lower: {error}"),
        }
    }
}

impl std::error::Error for SourceTransactionError {}

#[derive(Debug)]
pub struct SourceEditPreview {
    pub module: String,
    pub previous_revision: SourceRevision,
    pub new_revision: SourceRevision,
    pub compiled: CompiledProgram,
    previous_source: String,
    previous_modules: Vec<String>,
}

impl SourceEditPreview {
    pub fn previous_source(&self) -> &str {
        &self.previous_source
    }

    pub fn source(&self) -> &str {
        self.compiled
            .sources()
            .get(&self.module)
            .expect("edited module belongs to the compiled source graph")
    }

    pub fn previous_modules(&self) -> &[String] {
        &self.previous_modules
    }
}

pub fn apply_source_edit_path(
    entry: &Path,
    batch: &SourceEditBatch,
) -> Result<SourceEditPreview, SourceTransactionError> {
    apply_source_edit_path_with_root(entry, batch).map(|(_, preview)| preview)
}

pub fn apply_source_edit_path_with_root(
    entry: &Path,
    batch: &SourceEditBatch,
) -> Result<(PathBuf, SourceEditPreview), SourceTransactionError> {
    let (root, compiled) =
        compile_path_with_root(entry).map_err(SourceTransactionError::Program)?;
    apply(&compiled, batch).map(|preview| (root, preview))
}

fn apply(
    compiled: &CompiledProgram,
    batch: &SourceEditBatch,
) -> Result<SourceEditPreview, SourceTransactionError> {
    let index = compiled
        .source_index()
        .map_err(SourceTransactionError::Program)?;
    validate_source_edit_batch(batch, index.revision(), &index)
        .map_err(SourceTransactionError::Contract)?;
    let mut module: Option<&str> = None;
    let mut replacements = Vec::new();
    for (operation_index, operation) in batch.operations.iter().enumerate() {
        let (target, site, expression) = match operation {
            crate::source_edit::SourceEditOperation::SetExpression {
                target,
                site,
                expression,
            } => (target, site, expression),
        };
        if module.is_some_and(|value| value != target.module.as_str()) {
            return Err(SourceTransactionError::MultipleModules);
        }
        module = Some(&target.module);
        let indexed =
            index
                .expression(target, site)
                .ok_or(SourceTransactionError::TargetNotFound {
                    operation: operation_index,
                })?;
        replacements.push(BorrowedTextEdit {
            range: indexed.range,
            replacement: &expression.source,
        });
    }
    let module = module.expect("validated source-edit operations are non-empty");
    let current = compiled
        .sources()
        .get(module)
        .ok_or(SourceTransactionError::TargetNotFound { operation: 0 })?;
    let previous_source = current.clone();
    let source = apply_borrowed_text_edits(current, replacements)
        .map_err(SourceTransactionError::Contract)?;
    let mut sources: std::collections::BTreeMap<String, String> = compiled
        .sources()
        .iter()
        .filter(|(path, _)| path.as_str() != module)
        .map(|(path, source)| (path.clone(), source.clone()))
        .collect();
    sources.insert(module.to_owned(), source);
    let next = compile_overlay(compiled.root_module(), sources)?;
    validate_semantics(&next)?;
    let new_revision = revision(&next)?;
    Ok(SourceEditPreview {
        module: module.to_owned(),
        previous_revision: index.revision().clone(),
        new_revision,
        compiled: next,
        previous_source,
        previous_modules: compiled.sources().keys().cloned().collect(),
    })
}

fn compile_overlay(
    root: &str,
    sources: std::collections::BTreeMap<String, String>,
) -> Result<CompiledProgram, SourceTransactionError> {
    let source = sources
        .get(root)
        .cloned()
        .ok_or(SourceTransactionError::TargetNotFound { operation: 0 })?;
    let loader = MemoryLoader::new(sources);
    compile::compile_with_loader(
        LoadedSource {
            id: root.to_owned(),
            source,
        },
        &loader,
    )
    .map_err(SourceTransactionError::Program)
}

fn validate_semantics(compiled: &CompiledProgram) -> Result<(), SourceTransactionError> {
    crate::authoring::lower_document(compiled.document())
        .map(|_| ())
        .map_err(SourceTransactionError::Lowering)
}

fn revision(compiled: &CompiledProgram) -> Result<SourceRevision, SourceTransactionError> {
    let modules = compiled
        .sources()
        .iter()
        .map(|(path, source)| SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    source_graph_revision(&modules).map_err(SourceTransactionError::Contract)
}
