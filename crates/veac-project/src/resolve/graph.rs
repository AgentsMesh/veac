use std::collections::{BTreeMap, BTreeSet};

use crate::{IssueCode, ProjectIssue, ResolvedTargetEdge, TargetInstance, TargetInstanceId};

pub(crate) fn check_delivery_conflicts(
    instances: &[TargetInstance],
    issues: &mut Vec<ProjectIssue>,
) {
    let mut destinations = BTreeMap::new();
    for instance in instances {
        for delivery in &instance.deliveries {
            let owner = format!("{}:{}", instance.id, delivery.id);
            if let Some(previous) =
                destinations.insert(delivery.destination.as_str(), owner.clone())
            {
                issues.push(ProjectIssue::new(
                    IssueCode::DeliveryPathConflict,
                    format!("instances.{}.deliveries.{}", instance.id, delivery.id),
                    format!(
                        "delivery path {:?} is also produced by {previous}",
                        delivery.destination.as_str()
                    ),
                ));
            }
        }
    }
}

pub(crate) fn build_order(
    instances: &[TargetInstance],
    edges: &[ResolvedTargetEdge],
    issues: &mut Vec<ProjectIssue>,
) -> Vec<TargetInstanceId> {
    let mut indegree: BTreeMap<_, usize> = instances
        .iter()
        .map(|instance| (instance.id.clone(), 0))
        .collect();
    let mut outgoing: BTreeMap<TargetInstanceId, BTreeSet<TargetInstanceId>> = BTreeMap::new();
    let mut endpoint_edges = BTreeSet::new();
    for edge in edges {
        if endpoint_edges.insert((edge.dependency.clone(), edge.consumer.clone())) {
            *indegree.get_mut(&edge.consumer).expect("known consumer") += 1;
            outgoing
                .entry(edge.dependency.clone())
                .or_default()
                .insert(edge.consumer.clone());
        }
    }
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut order = Vec::with_capacity(instances.len());
    while let Some(id) = ready.pop_first() {
        order.push(id.clone());
        for consumer in outgoing.get(&id).into_iter().flatten() {
            let count = indegree.get_mut(consumer).expect("known consumer");
            *count -= 1;
            if *count == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    if order.len() != instances.len() {
        let cycle: Vec<_> = indegree
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(id, _)| id.as_str())
            .collect();
        issues.push(ProjectIssue::new(
            IssueCode::DependencyCycle,
            "targets",
            format!("dependency cycle contains: {}", cycle.join(", ")),
        ));
    }
    order
}
