mod project_support;

use veac_artifact::ContentDigest;
use veac_build::*;
use veac_project::{DeliveryKind, ProjectPath, ProjectTargetEntry, RESOLVED_GRAPH_VERSION};

use project_support::{graph, Fixture};

#[test]
fn resolved_project_graph_becomes_a_closed_artifact_dag() {
    let fixture = Fixture::new();
    let plan = fixture.adapter().adapt(&graph()).unwrap();

    assert_eq!(plan.graph.len(), 3);
    assert_eq!(plan.graph.topology().len(), 3);
    assert_eq!(plan.nodes.len(), 3);
    assert_eq!(plan.deliveries.len(), 3);
    assert_eq!(plan.manifest_digest, ContentDigest::sha256(b"manifest"));
    assert!(plan
        .deliveries
        .windows(2)
        .all(|pair| pair[0].instance <= pair[1].instance));

    let mut kinds = Vec::new();
    let mut bound_roles = Vec::new();
    for node in plan.graph.topology() {
        let action = plan.graph.node(node).unwrap().action();
        kinds.push(action.kind_name());
        bound_roles.extend(
            action
                .computation()
                .bound_sources
                .iter()
                .map(|source| source.role),
        );
        assert_eq!(action.version(), PROJECT_ACTION_VERSION);
        assert!(!action.canonical_bytes().unwrap().is_empty());
    }
    kinds.sort();
    bound_roles.sort_by_key(|role| match role {
        ProjectSourceRole::Material => 0,
        ProjectSourceRole::AssetFact => 1,
    });
    assert_eq!(kinds, ["evidence", "media_derivation", "veac_render"]);
    assert_eq!(
        bound_roles,
        [ProjectSourceRole::Material, ProjectSourceRole::AssetFact]
    );
}

#[test]
fn cache_identity_tracks_inputs_but_excludes_delivery_destinations() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let original = graph();
    let baseline = adapter.adapt(&original).unwrap();

    let mut relocated = original.clone();
    relocated.instances[0].deliveries[0].destination = ProjectPath::new("elsewhere/final.bin");
    let relocated = adapter.adapt(&relocated).unwrap();
    assert_eq!(baseline.graph.digest(), relocated.graph.digest());

    std::fs::write(fixture.material.join("logo.bin"), b"changed logo").unwrap();
    let changed = adapter.adapt(&original).unwrap();
    assert_ne!(baseline.graph.digest(), changed.graph.digest());

    std::fs::write(
        fixture.source.join("plate-module.veac"),
        "module { export fn title() -> text { \"Changed\" } }\n",
    )
    .unwrap();
    let module_changed = adapter.adapt(&original).unwrap();
    assert_ne!(changed.graph.digest(), module_changed.graph.digest());

    let entry = std::fs::read_to_string(fixture.source.join("plate.veac")).unwrap();
    std::fs::write(fixture.source.join("plate.veac"), format!("{entry}\n")).unwrap();
    let source_changed = adapter.adapt(&original).unwrap();
    assert_ne!(module_changed.graph.digest(), source_changed.graph.digest());
}

#[test]
fn adapter_rejects_invalid_or_unsupported_resolved_contracts() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();

    let mut invalid = graph();
    invalid.version = RESOLVED_GRAPH_VERSION + 1;
    assert!(adapter
        .adapt(&invalid)
        .err()
        .unwrap()
        .message()
        .contains("version"));

    let mut invalid = graph();
    invalid.manifest_digest = ContentDigest::sha256(b"manifest").value;
    assert!(adapter
        .adapt(&invalid)
        .err()
        .unwrap()
        .message()
        .contains("scheme"));

    let mut invalid = graph();
    invalid.manifest_digest = "sha256:not-a-digest".to_owned();
    assert!(adapter
        .adapt(&invalid)
        .err()
        .unwrap()
        .message()
        .contains("digest"));

    let mut invalid = graph();
    invalid.instances[0].deliveries[0].kind = DeliveryKind::Directory;
    assert!(adapter
        .adapt(&invalid)
        .err()
        .unwrap()
        .message()
        .contains("directory"));

    let mut invalid = graph();
    invalid.instances[0].entry = ProjectTargetEntry::Evidence {
        contract: ProjectPath::new("missing.veac"),
    };
    assert!(adapter.adapt(&invalid).is_err());
}

#[test]
fn adapter_roots_and_sources_are_non_symlink_filesystem_authorities() {
    let fixture = Fixture::new();
    let file_root = fixture.temp.path().join("not-a-directory");
    std::fs::write(&file_root, b"file").unwrap();
    assert!(ProjectGraphAdapter::new(&file_root, &fixture.material).is_err());

    #[cfg(unix)]
    {
        let link = fixture.material.join("linked.bin");
        std::os::unix::fs::symlink(fixture.material.join("logo.bin"), &link).unwrap();
        let mut invalid = graph();
        let plate = invalid
            .instances
            .iter_mut()
            .find(|item| item.id.as_str() == "plate")
            .unwrap();
        let veac_project::ResolvedInputSource::ProjectMaterial { path } =
            &mut plate.inputs[0].source
        else {
            unreachable!()
        };
        *path = ProjectPath::new("linked.bin");
        assert!(fixture.adapter().adapt(&invalid).is_err());
    }
}
