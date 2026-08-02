use std::collections::BTreeMap;

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::source_edit::{ExpressionSite, SourceNodeRef, SourceRevision, TextRange};

use super::SourceIndex;

pub const SOURCE_INDEX_SCHEMA: &str = "https://veac.dev/schemas/source-index";
pub const SOURCE_INDEX_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexInventory {
    pub schema: String,
    pub schema_version: u32,
    pub revision: SourceRevision,
    pub nodes: Vec<SourceIndexNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexNode {
    pub target: SourceNodeRef,
    pub range: TextRange,
    pub expressions: Vec<SourceIndexExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexExpression {
    pub site: ExpressionSite,
    pub source: String,
    pub range: TextRange,
}

impl SourceIndex {
    /// Returns a detached, read-only view ordered by module, typed path, and expression site.
    pub fn inventory(&self) -> SourceIndexInventory {
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
        SourceIndexInventory {
            schema: SOURCE_INDEX_SCHEMA.to_owned(),
            schema_version: SOURCE_INDEX_SCHEMA_VERSION,
            revision: self.revision.clone(),
            nodes: nodes.into_values().collect(),
        }
    }
}

pub fn source_index_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(SourceIndexInventory))
}
