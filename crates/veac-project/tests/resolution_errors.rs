mod support;

use support::{manifest, target_ref};
use veac_project::*;

#[test]
fn same_selector_reports_no_match_when_consumer_lacks_required_axis() {
    let mut value = manifest();
    let ProjectInputSource::Artifact { target, .. } = &mut value.targets[1].inputs[0].source else {
        unreachable!()
    };
    target.selector = InstanceSelector::Same {};
    let error = resolve_manifest(&value).unwrap_err();
    assert!(support::codes(&error).contains(&IssueCode::SelectorNoMatch));
}

#[test]
fn exact_selector_must_select_one_instance() {
    let mut value = manifest();
    let ProjectInputSource::Artifact { target, .. } = &mut value.targets[1].inputs[0].source else {
        unreachable!()
    };
    *target = target_ref(
        "plate",
        None,
        InstanceSelector::Exact {
            locale: Some(LocaleId::from("zh-cn")),
            axes: Default::default(),
        },
    );
    let error = resolve_manifest(&value).unwrap_err();
    assert!(support::codes(&error).contains(&IssueCode::SelectorNotUnique));
}

#[test]
fn concrete_dependency_cycles_are_rejected() {
    let mut value = manifest();
    value.targets[0]
        .needs
        .push(target_ref("final", None, InstanceSelector::AllMatching {}));
    let error = resolve_manifest(&value).unwrap_err();
    let cycle = error
        .issues()
        .iter()
        .find(|issue| issue.code == IssueCode::DependencyCycle)
        .unwrap();
    assert!(cycle.message.contains("final@profile="));
    assert!(cycle.message.contains("plate@profile="));
}

#[test]
fn concrete_delivery_destinations_cannot_collide() {
    let mut value = manifest();
    value.targets[0].deliveries[0] = ProjectDelivery::File {
        id: DeliveryId::from("plate-file"),
        output: OutputId::from("video"),
        destination: DeliveryPathTemplate::new("plates/shared.mp4"),
    };
    let error = resolve_manifest(&value).unwrap_err();
    let conflicts: Vec<_> = error
        .issues()
        .iter()
        .filter(|issue| issue.code == IssueCode::DeliveryPathConflict)
        .collect();
    assert_eq!(conflicts.len(), 7);
    assert!(conflicts[0].message.contains("plates/shared.mp4"));
}

#[test]
fn resolution_errors_are_canonically_sorted_and_deduplicated() {
    let mut value = manifest();
    value.targets[0]
        .needs
        .push(target_ref("final", None, InstanceSelector::AllMatching {}));
    value.targets[0].deliveries[0] = ProjectDelivery::File {
        id: DeliveryId::from("plate-file"),
        output: OutputId::from("video"),
        destination: DeliveryPathTemplate::new("same.mp4"),
    };
    let error = resolve_manifest(&value).unwrap_err();
    assert!(error.issues().windows(2).all(|pair| {
        (&pair[0].path, pair[0].code, &pair[0].message)
            <= (&pair[1].path, pair[1].code, &pair[1].message)
    }));
}

#[test]
fn all_input_and_delivery_variants_resolve_to_closed_ir() {
    let mut value = manifest();
    value.targets[1].inputs = vec![
        ProjectInput {
            id: InputId::from("literal"),
            source: ProjectInputSource::Literal {
                value: ProjectLiteral::Integer { value: 42 },
            },
        },
        ProjectInput {
            id: InputId::from("asset"),
            source: ProjectInputSource::AssetFact {
                path: ProjectPath::new("host.mp4"),
                fact: FactId::from("duration"),
            },
        },
        ProjectInput {
            id: InputId::from("analysis"),
            source: ProjectInputSource::AnalysisFact {
                target: target_ref("plate", None, InstanceSelector::AllMatching {}),
                output: OutputId::from("video"),
                fact: FactId::from("motion"),
            },
        },
    ];
    value.targets[1].outputs.push(ProjectOutput::Directory {
        id: OutputId::from("frames"),
    });
    value.targets[1]
        .deliveries
        .push(ProjectDelivery::Directory {
            id: DeliveryId::from("frames-dir"),
            output: OutputId::from("frames"),
            destination: DeliveryPathTemplate::new("frames/{profile}/{locale}"),
        });
    let graph = resolve_manifest(&value).unwrap();
    let instance = graph
        .instances
        .iter()
        .find(|item| item.target.as_str() == "final")
        .unwrap();
    assert!(matches!(
        instance.inputs[0].source,
        ResolvedInputSource::AnalysisFact { .. }
    ));
    assert!(matches!(
        instance.inputs[1].source,
        ResolvedInputSource::AssetFact { .. }
    ));
    assert!(matches!(
        instance.inputs[2].source,
        ResolvedInputSource::Literal { .. }
    ));
    assert!(instance
        .deliveries
        .iter()
        .any(|item| item.kind == DeliveryKind::Directory));
}

#[test]
fn input_source_exposes_only_target_backed_references() {
    let value = manifest();
    assert!(value.targets[0].inputs[0].source.target_ref().is_none());
    let source = &value.targets[1].inputs[0].source;
    let (reference, output) = source.target_ref().unwrap();
    assert_eq!(reference.target.as_str(), "plate");
    assert_eq!(output.as_str(), "video");
}
