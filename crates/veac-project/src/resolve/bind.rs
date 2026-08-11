use std::collections::BTreeMap;

use crate::{
    DeliveryKind, ProjectDelivery, ProjectInput, ProjectInputSource, ProjectIssue,
    ProjectManifestV1, ResolvedDelivery, ResolvedInput, ResolvedInputSource, ResolvedTargetEdge,
    TargetId, TargetInstance, TargetRef,
};

use super::{push_selection_issue, select::select};

pub(crate) fn bind_instances(
    manifest: &ProjectManifestV1,
    index: &BTreeMap<TargetId, Vec<TargetInstance>>,
    instances: &mut [TargetInstance],
    edges: &mut Vec<ResolvedTargetEdge>,
    issues: &mut Vec<ProjectIssue>,
) {
    for instance in instances {
        let target = manifest
            .targets
            .iter()
            .find(|target| target.id == instance.target)
            .expect("validated instance target");
        for input in &target.inputs {
            if let Some(resolved) = bind_input(input, instance, index, edges, issues) {
                instance.inputs.push(resolved);
            }
        }
        instance
            .inputs
            .sort_by(|left, right| left.id.cmp(&right.id));
        for need in &target.needs {
            bind_need(need, instance, index, edges, issues);
        }
        instance.deliveries = target
            .deliveries
            .iter()
            .filter_map(|delivery| bind_delivery(delivery, instance))
            .collect();
        instance
            .deliveries
            .sort_by(|left, right| left.id.cmp(&right.id));
    }
}

fn bind_input(
    input: &ProjectInput,
    consumer: &TargetInstance,
    index: &BTreeMap<TargetId, Vec<TargetInstance>>,
    edges: &mut Vec<ResolvedTargetEdge>,
    issues: &mut Vec<ProjectIssue>,
) -> Option<ResolvedInput> {
    let source = match &input.source {
        ProjectInputSource::Literal { value } => ResolvedInputSource::Literal {
            value: value.clone(),
        },
        ProjectInputSource::ProfileBinding {} => ResolvedInputSource::ProfileBinding {
            profile: consumer
                .profile
                .clone()
                .expect("validated profile binding has a concrete profile"),
        },
        ProjectInputSource::LocaleBinding {} => ResolvedInputSource::LocaleBinding {
            locale: consumer
                .locale
                .clone()
                .expect("validated locale binding has a concrete locale"),
        },
        ProjectInputSource::MatrixBinding { axis } => ResolvedInputSource::MatrixBinding {
            axis: axis.clone(),
            value: consumer
                .matrix
                .get(axis)
                .cloned()
                .expect("validated matrix binding has a concrete value"),
        },
        ProjectInputSource::ProjectMaterial { path } => {
            ResolvedInputSource::ProjectMaterial { path: path.clone() }
        }
        ProjectInputSource::AssetFact { path, fact } => ResolvedInputSource::AssetFact {
            path: path.clone(),
            fact: fact.clone(),
        },
        ProjectInputSource::Artifact { target, output } => {
            let selected = bind_reference(target, consumer, index, issues)?;
            add_edges(&selected, consumer, Some(&input.id), Some(output), edges);
            ResolvedInputSource::Artifact {
                instances: selected.into_iter().map(|item| item.id).collect(),
                output: output.clone(),
            }
        }
        ProjectInputSource::AnalysisFact {
            target,
            output,
            fact,
        } => {
            let selected = bind_reference(target, consumer, index, issues)?;
            add_edges(&selected, consumer, Some(&input.id), Some(output), edges);
            ResolvedInputSource::AnalysisFact {
                instances: selected.into_iter().map(|item| item.id).collect(),
                output: output.clone(),
                fact: fact.clone(),
            }
        }
    };
    Some(ResolvedInput {
        id: input.id.clone(),
        source,
    })
}

fn bind_need(
    reference: &TargetRef,
    consumer: &TargetInstance,
    index: &BTreeMap<TargetId, Vec<TargetInstance>>,
    edges: &mut Vec<ResolvedTargetEdge>,
    issues: &mut Vec<ProjectIssue>,
) {
    if let Some(selected) = bind_reference(reference, consumer, index, issues) {
        add_edges(&selected, consumer, None, None, edges);
    }
}

fn bind_reference(
    reference: &TargetRef,
    consumer: &TargetInstance,
    index: &BTreeMap<TargetId, Vec<TargetInstance>>,
    issues: &mut Vec<ProjectIssue>,
) -> Option<Vec<TargetInstance>> {
    let candidates = index.get(&reference.target).expect("validated target ref");
    match select(reference, consumer, candidates) {
        Ok(selected) => Some(selected),
        Err((code, message)) => {
            push_selection_issue(code, &consumer.id, message, issues);
            None
        }
    }
}

fn add_edges(
    dependencies: &[TargetInstance],
    consumer: &TargetInstance,
    binding: Option<&crate::InputId>,
    output: Option<&crate::OutputId>,
    edges: &mut Vec<ResolvedTargetEdge>,
) {
    edges.extend(dependencies.iter().map(|dependency| ResolvedTargetEdge {
        dependency: dependency.id.clone(),
        consumer: consumer.id.clone(),
        binding: binding.cloned(),
        output: output.cloned(),
    }));
}

fn bind_delivery(
    delivery: &ProjectDelivery,
    instance: &TargetInstance,
) -> Option<ResolvedDelivery> {
    let destination =
        super::super::validate::expand_template(delivery.destination().as_str(), instance)?;
    let kind = match delivery {
        ProjectDelivery::File { .. } => DeliveryKind::File,
        ProjectDelivery::Directory { .. } => DeliveryKind::Directory,
    };
    Some(ResolvedDelivery {
        id: delivery.id().clone(),
        output: delivery.output().clone(),
        kind,
        destination: crate::ProjectPath::new(destination),
    })
}
