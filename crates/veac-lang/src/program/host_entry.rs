use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::diagnostic::{Diagnostic, Diagnostics};
use super::expression::{self, CompiledFunction, ExecutionBudget, FunctionMap, Value};
use super::loader::{FileSystemLoader, LoadedSource, MemoryLoader, SourceLoader};
use super::resolve;
use super::{CompilerDatabase, EntryContract, PreparedSourceGraph, TypeRegistry};
use crate::authoring::Span;

#[derive(Debug, Clone)]
pub struct PreparedHostEntry {
    source_graph: PreparedSourceGraph,
    entry: Arc<CompiledFunction>,
    functions: Arc<FunctionMap>,
    types: Arc<TypeRegistry>,
}

#[derive(Debug, Clone)]
pub struct EvaluatedHostEntry {
    value: Value,
    types: Arc<TypeRegistry>,
}

pub fn prepare_host_path(
    path: &Path,
    contract: &EntryContract,
) -> Result<(PathBuf, PreparedHostEntry), Diagnostics> {
    let (loader, entry) =
        FileSystemLoader::for_entry(path).map_err(|message| load(path, message))?;
    let root = loader.root().to_owned();
    prepare_host_with_loader(entry, &loader, contract).map(|value| (root, value))
}

pub fn prepare_host_root_path(
    root: &Path,
    entry: &Path,
    contract: &EntryContract,
) -> Result<PreparedHostEntry, Diagnostics> {
    let (loader, source) = FileSystemLoader::for_root_entry(root, entry)
        .map_err(|message| load(&root.join(entry), message))?;
    prepare_host_with_loader(source, &loader, contract)
}

pub fn prepare_host_source(
    source: &str,
    contract: &EntryContract,
) -> Result<PreparedHostEntry, Diagnostics> {
    super::limits::check_source_size("main.veac", source, Span::default())
        .map_err(Diagnostics::one)?;
    prepare_host_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: source.to_owned(),
        },
        &MemoryLoader::default(),
        contract,
    )
}

pub fn prepare_host_with_loader(
    entry: LoadedSource,
    loader: &dyn SourceLoader,
    contract: &EntryContract,
) -> Result<PreparedHostEntry, Diagnostics> {
    let database = CompilerDatabase::default();
    let source_graph = database
        .prepare_source_graph(entry, loader, contract.preludes())
        .map_err(Diagnostics)?;
    let resolution = resolve::contract_entry_with_database(
        source_graph.root(),
        &source_graph,
        &ExecutionBudget::default(),
        contract,
        &database,
    )
    .map_err(Diagnostics)?;
    let function = resolution
        .scope
        .functions
        .lookup(contract.function())
        .expect("validated host entry is resolved")
        .clone();
    contract
        .validate_compiled(&function, &resolution.scope.types)
        .map_err(Diagnostics::one)?;
    Ok(PreparedHostEntry {
        source_graph,
        entry: function,
        functions: resolution.scope.functions,
        types: resolution.scope.types,
    })
}

impl PreparedHostEntry {
    pub fn execute(&self, arguments: &[Value]) -> Result<EvaluatedHostEntry, Diagnostics> {
        let environment = expression::Environment::new();
        let value = expression::runtime::execute_value_entry(
            &self.entry,
            &self.functions,
            &self.types,
            arguments,
            &environment,
            &ExecutionBudget::default(),
        )
        .map_err(|error| runtime_error(self.root_module(), &self.entry, error))?;
        Ok(EvaluatedHostEntry {
            value,
            types: Arc::clone(&self.types),
        })
    }

    pub fn root_module(&self) -> &str {
        self.source_graph.root_module()
    }

    pub fn sources(&self) -> &std::collections::BTreeMap<String, String> {
        self.source_graph.sources()
    }

    pub fn source_graph(&self) -> &PreparedSourceGraph {
        &self.source_graph
    }

    pub fn type_registry(&self) -> &TypeRegistry {
        &self.types
    }
}

impl EvaluatedHostEntry {
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn type_registry(&self) -> &TypeRegistry {
        &self.types
    }
}

fn load(path: &Path, message: String) -> Diagnostics {
    Diagnostics::one(Diagnostic::new(
        "PROGRAM_ENTRY_LOAD",
        path.display().to_string(),
        message,
        Span::default(),
    ))
}

fn runtime_error(
    path: &str,
    entry: &CompiledFunction,
    error: expression::ExpressionError,
) -> Diagnostics {
    let range = entry.origin().map_or(0..0, |value| value.body_span());
    Diagnostics::one(super::expression_diagnostic::runtime(
        "PROGRAM_ENTRY_RUNTIME",
        path,
        Span {
            start: range.start,
            end: range.end,
        },
        error,
    ))
}
