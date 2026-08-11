mod support;

use support::{manifest, target_ref};
use veac_project::*;

#[test]
fn resolver_expands_a_sorted_concrete_graph() {
    let graph = resolve_manifest(&manifest()).unwrap();
    assert_eq!(graph.version, 1);
    assert_eq!(graph.instances.len(), 12);
    assert_eq!(graph.edges.len(), 8);
    assert_eq!(graph.build_order.len(), 12);
    assert!(graph.manifest_digest.starts_with("sha256:"));
    assert!(graph
        .instances
        .windows(2)
        .all(|pair| pair[0].id < pair[1].id));
    assert!(graph.edges.windows(2).all(|pair| pair[0] <= pair[1]));
    let final_instance = graph
        .instances
        .iter()
        .find(|instance| instance.id.as_str() == "final@profile=preview@locale=zh-cn")
        .unwrap();
    assert_eq!(final_instance.inputs.len(), 1);
    assert_eq!(
        final_instance.deliveries[0].destination.as_str(),
        "final/preview/zh-cn.mp4"
    );
    let ResolvedInputSource::Artifact { instances, output } = &final_instance.inputs[0].source
    else {
        panic!("artifact binding expected");
    };
    assert_eq!(instances.len(), 2);
    assert_eq!(output.as_str(), "video");
    assert!(instances
        .iter()
        .all(|id| id.as_str().contains("@locale=zh-cn")));
}

#[test]
fn same_selector_tracks_every_consumer_dimension() {
    let mut value = manifest();
    value.targets[1].axes = value.targets[0].axes.clone();
    value.targets[1].deliveries[0] = ProjectDelivery::File {
        id: DeliveryId::from("final-file"),
        output: OutputId::from("video"),
        destination: DeliveryPathTemplate::new("final/{profile}/{locale}/{axis.theme}.mp4"),
    };
    let ProjectInputSource::Artifact { target, .. } = &mut value.targets[1].inputs[0].source else {
        unreachable!()
    };
    target.selector = InstanceSelector::Same {};
    let graph = resolve_manifest(&value).unwrap();
    assert_eq!(graph.instances.len(), 16);
    assert_eq!(graph.edges.len(), 8);
    for instance in graph
        .instances
        .iter()
        .filter(|item| item.target.as_str() == "final")
    {
        let ResolvedInputSource::Artifact { instances, .. } = &instance.inputs[0].source else {
            unreachable!()
        };
        assert_eq!(instances.len(), 1);
        assert_eq!(
            instances[0].as_str().replace("plate@", "final@"),
            instance.id.as_str()
        );
    }
}

#[test]
fn exact_selector_can_pin_profile_locale_and_axes() {
    let mut value = manifest();
    let ProjectInputSource::Artifact { target, .. } = &mut value.targets[1].inputs[0].source else {
        unreachable!()
    };
    *target = target_ref(
        "plate",
        Some("preview"),
        InstanceSelector::Exact {
            locale: Some(LocaleId::from("zh-cn")),
            axes: [(AxisId::from("theme"), AxisValue::from("dark"))]
                .into_iter()
                .collect(),
        },
    );
    let graph = resolve_manifest(&value).unwrap();
    for instance in graph
        .instances
        .iter()
        .filter(|item| item.target.as_str() == "final")
    {
        let ResolvedInputSource::Artifact { instances, .. } = &instance.inputs[0].source else {
            unreachable!()
        };
        assert_eq!(
            instances,
            &[TargetInstanceId::from(
                "plate@profile=preview@locale=zh-cn@theme=dark"
            )]
        );
    }
}

#[test]
fn needs_create_ordering_edges_without_bindings() {
    let mut value = manifest();
    value.targets[1]
        .needs
        .push(target_ref("plate", None, InstanceSelector::AllMatching {}));
    let graph = resolve_manifest(&value).unwrap();
    assert_eq!(graph.edges.len(), 16);
    assert_eq!(
        graph
            .edges
            .iter()
            .filter(|edge| edge.binding.is_none())
            .count(),
        8
    );
    assert!(graph
        .edges
        .iter()
        .filter(|edge| edge.binding.is_none())
        .all(|edge| edge.output.is_none()));
    for edge in &graph.edges {
        let dependency = graph
            .build_order
            .iter()
            .position(|id| id == &edge.dependency)
            .unwrap();
        let consumer = graph
            .build_order
            .iter()
            .position(|id| id == &edge.consumer)
            .unwrap();
        assert!(dependency < consumer);
    }
}

#[test]
fn default_profile_applies_without_profile_inheritance() {
    let mut value = manifest();
    value.targets.truncate(1);
    value.targets[0].profiles.clear();
    let graph = resolve_manifest(&value).unwrap();
    assert_eq!(graph.instances.len(), 4);
    assert!(graph
        .instances
        .iter()
        .all(|item| item.profile.as_ref().unwrap().as_str() == "preview"));

    value.defaults.profile = None;
    value.targets[0].deliveries[0] = ProjectDelivery::File {
        id: DeliveryId::from("plate-file"),
        output: OutputId::from("video"),
        destination: DeliveryPathTemplate::new("plates/{locale}/{axis.theme}.mp4"),
    };
    let graph = resolve_manifest(&value).unwrap();
    assert!(graph.instances.iter().all(|item| item.profile.is_none()));
}

#[test]
fn unprofiled_and_unlocalized_dependencies_match_profiled_consumers() {
    let mut value = manifest();
    value.defaults.profile = None;
    value.targets[0].profiles.clear();
    value.targets[0].localized = false;
    value.targets[0].axes.clear();
    value.targets[0].deliveries[0] = ProjectDelivery::File {
        id: DeliveryId::from("plate-file"),
        output: OutputId::from("video"),
        destination: DeliveryPathTemplate::new("plates/base.mp4"),
    };
    let ProjectInputSource::Artifact { target, .. } = &mut value.targets[1].inputs[0].source else {
        unreachable!()
    };
    target.selector = InstanceSelector::Same {};
    let graph = resolve_manifest(&value).unwrap();
    assert_eq!(graph.instances.len(), 5);
    assert_eq!(graph.edges.len(), 4);
}
