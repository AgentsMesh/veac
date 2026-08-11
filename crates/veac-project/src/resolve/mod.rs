mod bind;
mod expand;
mod graph;
mod select;

use std::collections::BTreeMap;

use crate::{
    manifest_digest, validate_manifest, IssueCode, ProjectIssue, ProjectIssues, ProjectManifestV1,
    ResolvedTargetGraph, TargetInstance, TargetInstanceId, RESOLVED_GRAPH_VERSION,
};

pub fn resolve_manifest(
    manifest: &ProjectManifestV1,
) -> Result<ResolvedTargetGraph, ProjectIssues> {
    validate_manifest(manifest)?;
    let digest = manifest_digest(manifest).map_err(serialization_issue)?;
    let mut instances = expand::expand_instances(manifest);
    let index = instance_index(&instances);
    let mut edges = Vec::new();
    let mut issues = Vec::new();
    bind::bind_instances(manifest, &index, &mut instances, &mut edges, &mut issues);
    edges.sort();
    edges.dedup();
    graph::check_delivery_conflicts(&instances, &mut issues);
    let build_order = graph::build_order(&instances, &edges, &mut issues);
    if !issues.is_empty() {
        return Err(ProjectIssues::sorted(issues));
    }
    Ok(ResolvedTargetGraph {
        version: RESOLVED_GRAPH_VERSION,
        manifest_digest: digest,
        instances,
        edges,
        build_order,
    })
}

fn instance_index(instances: &[TargetInstance]) -> BTreeMap<crate::TargetId, Vec<TargetInstance>> {
    let mut index: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for instance in instances {
        index
            .entry(instance.target.clone())
            .or_default()
            .push(instance.clone());
    }
    index
}

fn serialization_issue(error: serde_json::Error) -> ProjectIssues {
    ProjectIssues::sorted(vec![ProjectIssue::new(
        IssueCode::Serialization,
        "manifest",
        format!("manifest cannot be canonicalized: {error}"),
    )])
}

pub(crate) fn push_selection_issue(
    code: IssueCode,
    consumer: &TargetInstanceId,
    message: String,
    issues: &mut Vec<ProjectIssue>,
) {
    issues.push(ProjectIssue::new(
        code,
        format!("instances.{consumer}.dependencies"),
        message,
    ));
}
