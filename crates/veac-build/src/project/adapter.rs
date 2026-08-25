use std::{collections::BTreeMap, path::PathBuf};

use veac_artifact::ContentDigest;
use veac_project::{
    ResolvedDelivery, ResolvedTargetGraph, TargetInstanceId, RESOLVED_GRAPH_VERSION,
};

use super::snapshot::ProjectRoots;
use crate::{
    BuildError, BuildResult, GraphBuilder, NodeId, NodeSpec, ProjectAction, ValidatedGraph,
};

mod action;
mod edges;
mod planning;

use action::ActionSources;

pub struct ProjectArtifact;

pub struct ProjectBuildPlan {
    pub graph: ValidatedGraph<ProjectAction>,
    pub manifest_digest: ContentDigest,
    pub nodes: BTreeMap<NodeId, ProjectNodeIdentity>,
    pub deliveries: Vec<PlannedDelivery>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectNodeIdentity {
    pub instance: TargetInstanceId,
    pub target: veac_project::TargetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedDelivery {
    pub node: NodeId,
    pub instance: TargetInstanceId,
    pub delivery: ResolvedDelivery,
}

pub struct ProjectGraphAdapter {
    roots: ProjectRoots,
}

impl ProjectGraphAdapter {
    pub fn new(
        source_base: impl Into<PathBuf>,
        material_root: impl Into<PathBuf>,
        packages: crate::ProjectPackageSet,
    ) -> BuildResult<Self> {
        Ok(Self {
            roots: ProjectRoots::new(source_base, material_root, packages)?,
        })
    }

    pub fn adapt(&self, resolved: &ResolvedTargetGraph) -> BuildResult<ProjectBuildPlan> {
        self.roots.revalidate_packages()?;
        if resolved.version != RESOLVED_GRAPH_VERSION {
            return Err(BuildError::invalid(
                "unsupported resolved project graph version",
            ));
        }
        let manifest_digest = planning::parse_digest(&resolved.manifest_digest)?;
        let mut specs = BTreeMap::new();
        let mut node_ids = BTreeMap::new();
        let mut identities = BTreeMap::new();
        let mut sources = ActionSources::default();
        for instance in &resolved.instances {
            let node_id = edges::instance_node_id(&instance.id)?;
            if identities.contains_key(&node_id) {
                return Err(BuildError::invalid(
                    "project instance node digest collision",
                ));
            }
            let action = self.action(instance, &mut sources)?;
            let mut spec = NodeSpec::new(node_id.clone(), action);
            for output in &instance.outputs {
                spec = spec.output(&edges::output_slot(output)?);
            }
            node_ids.insert(instance.id.clone(), node_id.clone());
            identities.insert(
                node_id,
                ProjectNodeIdentity {
                    instance: instance.id.clone(),
                    target: instance.target.clone(),
                },
            );
            specs.insert(instance.id.clone(), spec);
        }
        for edge in &resolved.edges {
            edges::bind(edge, &node_ids, &mut specs)?;
        }
        let mut builder = GraphBuilder::new();
        for (_, spec) in specs {
            builder.add_node(spec)?;
        }
        let graph = builder.validate()?;
        let mut deliveries = planning::deliveries(resolved, &node_ids)?;
        deliveries.sort_by(|left, right| {
            (&left.instance, &left.delivery.id).cmp(&(&right.instance, &right.delivery.id))
        });
        self.roots.revalidate_packages()?;
        Ok(ProjectBuildPlan {
            graph,
            manifest_digest,
            nodes: identities,
            deliveries,
        })
    }
}
