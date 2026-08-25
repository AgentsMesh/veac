mod build_inputs;
mod model;

use std::collections::BTreeMap;

use schemars::schema_for;

use crate::source_edit::TextRange;

use super::SourceIndex;
pub use model::*;

pub const SOURCE_INDEX_SCHEMA: &str = "https://veac.dev/schemas/source-index";
pub const SOURCE_INDEX_SCHEMA_VERSION: u32 = 11;

pub(crate) use build_inputs::describe as describe_build_inputs;

impl SourceIndex {
    pub(crate) fn with_build_inputs(mut self, values: Vec<SourceIndexBuildInput>) -> Self {
        self.build_inputs = values;
        self
    }

    /// Returns a detached view ordered by module, typed path, and source site.
    pub fn inventory(
        &self,
        revision: &crate::source_edit::SourceRevision,
    ) -> Result<SourceIndexInventory, crate::source_edit::SourceEditError> {
        let revision = validate_revision(self, revision)?;
        let modules = self
            .modules
            .iter()
            .map(|(module, range)| SourceIndexModule {
                module: module.clone(),
                range: *range,
                imports: self.module_imports(module),
                declarations: self.module_declarations(module),
            })
            .collect();
        let mut nodes = self
            .nodes
            .iter()
            .map(|(target, span)| {
                let node = SourceIndexNode {
                    target: target.clone(),
                    range: TextRange {
                        start: span.start,
                        end: span.end,
                    },
                    expressions: Vec::new(),
                    statements: Vec::new(),
                    bodies: Vec::new(),
                    declarations: Vec::new(),
                };
                (target.clone(), node)
            })
            .collect::<BTreeMap<_, _>>();
        for ((target, site), value) in &self.expressions {
            nodes
                .get_mut(target)
                .expect("indexed expressions belong to registered nodes")
                .expressions
                .push(SourceIndexExpression {
                    site: site.clone(),
                    source: value.source.clone(),
                    range: value.range,
                });
        }
        for ((target, site), value) in &self.bodies {
            nodes
                .get_mut(target)
                .expect("indexed bodies belong to registered nodes")
                .bodies
                .push(SourceIndexBody {
                    site: *site,
                    source: value.source.clone(),
                    range: value.range,
                });
        }
        for ((target, site), value) in &self.statements {
            nodes
                .get_mut(target)
                .expect("indexed statements belong to registered nodes")
                .statements
                .push(SourceIndexStatement {
                    site: site.clone(),
                    source: value.source.clone(),
                    range: value.range,
                });
        }
        for ((target, site), value) in &self.declarations {
            nodes
                .get_mut(target)
                .expect("indexed declarations belong to registered nodes")
                .declarations
                .push(SourceIndexDeclaration {
                    site: *site,
                    source: value.source.clone(),
                    range: value.range,
                });
        }
        Ok(SourceIndexInventory {
            schema: SOURCE_INDEX_SCHEMA.to_owned(),
            schema_version: SOURCE_INDEX_SCHEMA_VERSION,
            revision,
            build_inputs: self.build_inputs.clone(),
            modules,
            nodes: nodes.into_values().collect(),
        })
    }

    fn module_imports(&self, module: &str) -> Vec<SourceIndexImport> {
        self.imports
            .iter()
            .filter(|(target, _)| target.module == module)
            .map(|(target, value)| SourceIndexImport {
                target: target.clone(),
                path: value.path.clone(),
                source: value.source.clone(),
                range: value.range,
            })
            .collect()
    }

    fn module_declarations(&self, module: &str) -> Vec<SourceIndexTopLevelDeclaration> {
        self.top_levels
            .iter()
            .filter(|(target, _)| target.module == module)
            .map(|(target, value)| SourceIndexTopLevelDeclaration {
                target: target.clone(),
                source: value.source.clone(),
                range: value.range,
            })
            .collect()
    }
}

fn validate_revision(
    index: &SourceIndex,
    revision: &crate::source_edit::SourceRevision,
) -> Result<crate::source_edit::SourceRevision, crate::source_edit::SourceEditError> {
    for digest in [
        &revision.authored_source_graph_sha256,
        &revision.complete_source_graph_sha256,
    ] {
        if !crate::source_edit::valid_sha256(digest) {
            return Err(crate::source_edit::SourceEditError::InvalidDigest(
                digest.clone(),
            ));
        }
    }
    let expected = index
        .bound_revision()
        .ok_or(crate::source_edit::SourceEditError::UnboundSourceIndex)?;
    if revision != &expected {
        return Err(crate::source_edit::SourceEditError::StaleRevision {
            expected: describe(&expected),
            actual: describe(revision),
        });
    }
    Ok(expected)
}

fn describe(revision: &crate::source_edit::SourceRevision) -> String {
    format!(
        "authored={}, complete={}",
        revision.authored_source_graph_sha256, revision.complete_source_graph_sha256
    )
}

pub fn source_index_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(SourceIndexInventory))
}
