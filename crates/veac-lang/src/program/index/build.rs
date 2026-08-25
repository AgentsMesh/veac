use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::source_edit::{source_graph_revision, SourceModule, SourceRevision};

use super::super::diagnostic::{Diagnostic, Diagnostics};
use super::super::parser;
use super::SourceIndex;
use crate::program::PreparedSourceGraph;

impl SourceIndex {
    /// Builds a publishable index from one immutable, fully discovered graph.
    pub fn build(graph: &PreparedSourceGraph) -> Result<Self, Diagnostics> {
        let mut index = Self::build_snapshot(&graph.project_sources())?;
        index.complete_revision = Some(graph.complete_revision());
        Ok(index)
    }

    /// Builds lookup state only; without a frozen graph identity it cannot publish inventory.
    pub(crate) fn build_snapshot(sources: &BTreeMap<String, String>) -> Result<Self, Diagnostics> {
        let modules = sources
            .iter()
            .map(|(path, source)| SourceModule::utf8(path, source))
            .collect::<Vec<_>>();
        let revision = source_graph_revision(&modules).map_err(|error| {
            Diagnostics::one(Diagnostic::new(
                "SOURCE_GRAPH_REVISION",
                "<source-graph>",
                error.to_string(),
                Span::default(),
            ))
        })?;
        let mut index = Self {
            revision,
            complete_revision: None,
            build_inputs: Vec::new(),
            expressions: BTreeMap::new(),
            statements: BTreeMap::new(),
            bodies: BTreeMap::new(),
            declarations: BTreeMap::new(),
            imports: BTreeMap::new(),
            modules: BTreeMap::new(),
            nodes: BTreeMap::new(),
            top_levels: BTreeMap::new(),
        };
        for (path, source) in sources {
            let file = parser::parse_executable(path, source).map_err(Diagnostics)?;
            index.file(&file).map_err(Diagnostics::one)?;
        }
        Ok(index)
    }

    pub(crate) fn bound_revision(&self) -> Option<SourceRevision> {
        self.complete_revision
            .as_ref()
            .map(|complete| SourceRevision::new(&self.revision, complete.sha256()))
    }
}
