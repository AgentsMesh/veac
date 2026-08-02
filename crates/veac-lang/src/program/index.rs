mod component;
mod inventory;
mod item;
mod preset;
mod project;
pub(crate) mod syntax;

pub use inventory::*;
pub(crate) use syntax::validate_expression_fragment;

use std::collections::BTreeMap;

use crate::authoring::Span;
use crate::source_edit::{
    source_graph_revision, ExpressionSite, SourceModule, SourceNodeRef, SourceRevision,
    SourceSnapshot, TextRange,
};

use super::diagnostic::{Diagnostic, Diagnostics};
use super::model::SurfaceFile;
use super::parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedExpression {
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone)]
pub struct SourceIndex {
    revision: SourceRevision,
    expressions: BTreeMap<(SourceNodeRef, ExpressionSite), IndexedExpression>,
    nodes: BTreeMap<SourceNodeRef, Span>,
}

impl SourceIndex {
    pub(crate) fn build(sources: &BTreeMap<String, String>) -> Result<Self, Diagnostics> {
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
            expressions: BTreeMap::new(),
            nodes: BTreeMap::new(),
        };
        for (path, source) in sources {
            let file = parser::parse(path, source).map_err(Diagnostics)?;
            index.file(&file).map_err(Diagnostics::one)?;
        }
        Ok(index)
    }

    pub fn revision(&self) -> &SourceRevision {
        &self.revision
    }

    pub fn expression(
        &self,
        target: &SourceNodeRef,
        site: &ExpressionSite,
    ) -> Option<&IndexedExpression> {
        self.expressions.get(&(target.clone(), site.clone()))
    }

    fn file(&mut self, file: &SurfaceFile) -> Result<(), Diagnostic> {
        for value in &file.constants {
            let target = SourceNodeRef::constant(&file.path, &value.name);
            self.register(&file.path, target.clone(), value.span)?;
            self.insert(
                &file.path,
                target,
                ExpressionSite::ConstantValue,
                &value.expression,
                value.expression_span,
            )?;
        }
        for value in &file.components {
            let target = SourceNodeRef::component(&file.path, &value.name);
            self.register(&file.path, target.clone(), value.span)?;
            for parameter in &value.parameters {
                if let Some(default) = &parameter.default {
                    self.insert(
                        &file.path,
                        target.clone(),
                        ExpressionSite::ComponentParameterDefault {
                            parameter: parameter.name.clone(),
                        },
                        &default.source,
                        default.span,
                    )?;
                }
            }
            component::index(self, file, value)?;
        }
        for value in &file.presets {
            preset::index(self, file, value)?;
        }
        for value in &file.instances {
            let target = SourceNodeRef::component_instance(&file.path, &value.id);
            self.register(&file.path, target.clone(), value.span)?;
            for (parameter, binding) in &value.bindings {
                self.insert(
                    &file.path,
                    target.clone(),
                    ExpressionSite::ComponentInstanceArgument {
                        parameter: parameter.clone(),
                    },
                    &binding.source,
                    binding.span,
                )?;
            }
        }
        project::index(self, file)
    }

    fn register(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self.nodes.insert(target.clone(), span).is_some() {
            return Err(ambiguous(path, &target, span));
        }
        Ok(())
    }

    fn insert(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        site: ExpressionSite,
        source: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value = IndexedExpression {
            source: source.to_owned(),
            range: TextRange {
                start: span.start,
                end: span.end,
            },
        };
        if self
            .expressions
            .insert((target.clone(), site), value)
            .is_some()
        {
            return Err(ambiguous(path, &target, span));
        }
        Ok(())
    }
}

impl SourceSnapshot for SourceIndex {
    fn node_exists(&self, target: &SourceNodeRef) -> bool {
        self.nodes.contains_key(target)
    }

    fn expression_source(&self, target: &SourceNodeRef, site: &ExpressionSite) -> Option<&str> {
        self.expression(target, site)
            .map(|value| value.source.as_str())
    }
}

fn ambiguous(path: &str, target: &SourceNodeRef, span: Span) -> Diagnostic {
    Diagnostic::new(
        "SOURCE_INDEX_AMBIGUOUS_TARGET",
        path,
        format!("source target {:?} is ambiguous", target.path),
        span,
    )
}

#[cfg(test)]
#[path = "index/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "index/syntax_tests.rs"]
mod syntax_tests;

#[cfg(test)]
#[path = "index/inventory_tests.rs"]
mod inventory_tests;
