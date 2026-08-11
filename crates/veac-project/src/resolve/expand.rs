use std::collections::BTreeMap;

use crate::{
    MatrixAssignment, ProfileId, ProjectManifestV1, ProjectTarget, TargetInstance, TargetInstanceId,
};

pub(crate) fn expand_instances(manifest: &ProjectManifestV1) -> Vec<TargetInstance> {
    let mut instances = Vec::new();
    for target in &manifest.targets {
        let profiles = profiles(manifest, target);
        let locales = locales(manifest, target);
        let assignments = assignments(target);
        for profile in &profiles {
            for locale in &locales {
                for matrix in &assignments {
                    let mut outputs = target.outputs.clone();
                    outputs.sort_by(|left, right| left.id().cmp(right.id()));
                    instances.push(TargetInstance {
                        id: instance_id(target, profile.as_ref(), locale.as_ref(), matrix),
                        target: target.id.clone(),
                        profile: profile.clone(),
                        locale: locale.clone(),
                        matrix: matrix.clone(),
                        entry: target.entry.clone(),
                        inputs: Vec::new(),
                        outputs,
                        deliveries: Vec::new(),
                    });
                }
            }
        }
    }
    instances.sort_by(|left, right| left.id.cmp(&right.id));
    instances
}

fn profiles(manifest: &ProjectManifestV1, target: &ProjectTarget) -> Vec<Option<ProfileId>> {
    let mut values = if target.profiles.is_empty() {
        vec![manifest.defaults.profile.clone()]
    } else {
        target.profiles.iter().cloned().map(Some).collect()
    };
    values.sort();
    values
}

fn locales(manifest: &ProjectManifestV1, target: &ProjectTarget) -> Vec<Option<crate::LocaleId>> {
    if !target.localized {
        return vec![None];
    }
    let mut values: Vec<_> = manifest
        .locales
        .iter()
        .map(|locale| Some(locale.id.clone()))
        .collect();
    values.sort();
    values
}

fn assignments(target: &ProjectTarget) -> Vec<MatrixAssignment> {
    let mut axes = target.axes.clone();
    axes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut output = vec![BTreeMap::new()];
    for axis in axes {
        let mut values = axis.values;
        values.sort();
        let mut next = Vec::with_capacity(output.len().saturating_mul(values.len()));
        for assignment in output {
            for value in &values {
                let mut expanded = assignment.clone();
                expanded.insert(axis.id.clone(), value.clone());
                next.push(expanded);
            }
        }
        output = next;
    }
    output
}

fn instance_id(
    target: &ProjectTarget,
    profile: Option<&ProfileId>,
    locale: Option<&crate::LocaleId>,
    matrix: &MatrixAssignment,
) -> TargetInstanceId {
    let mut output = target.id.to_string();
    if let Some(profile) = profile {
        output.push_str("@profile=");
        output.push_str(profile.as_str());
    }
    if let Some(locale) = locale {
        output.push_str("@locale=");
        output.push_str(locale.as_str());
    }
    for (axis, value) in matrix {
        output.push('@');
        output.push_str(axis.as_str());
        output.push('=');
        output.push_str(value.as_str());
    }
    TargetInstanceId::new(output)
}
