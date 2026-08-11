use std::fs;

use veac_project::{
    build_project_path, build_project_root_path, build_project_source, resolve_manifest,
    InstanceSelector, ProjectAuthoringError, ProjectInputSource, ProjectTargetEntry,
};

const SOURCE: &str = include_str!("fixtures/authored_complete.veac");
const DERIVATION: &str = include_str!("fixtures/authored_derivation.veac");

#[test]
fn authored_workspace_decodes_the_complete_closed_contract() {
    let authored = build_project_source(SOURCE).unwrap();
    let manifest = &authored.manifest;
    assert_eq!(manifest.profiles.len(), 3);
    assert_eq!(manifest.targets.len(), 3);
    assert!(matches!(
        manifest.targets[2].entry,
        ProjectTargetEntry::Evidence { .. }
    ));
    assert_eq!(resolve_manifest(manifest).unwrap().instances.len(), 3);
}

#[test]
fn authored_references_decode_same_exact_and_all_matching() {
    let authored = build_project_source(SOURCE).unwrap();
    let consumer = &authored.manifest.targets[1];
    let ProjectInputSource::Artifact { target, .. } = &consumer.inputs[0].source else {
        panic!("expected artifact input");
    };
    assert!(matches!(target.selector, InstanceSelector::Same {}));
    let ProjectInputSource::AnalysisFact { target, .. } = &consumer.inputs[1].source else {
        panic!("expected analysis input");
    };
    assert!(matches!(target.selector, InstanceSelector::Exact { .. }));
    assert!(matches!(
        consumer.needs[0].selector,
        InstanceSelector::AllMatching {}
    ));
}

#[test]
fn filesystem_entry_points_preserve_the_project_root() {
    let root = std::env::temp_dir().join(format!("veac-project-{}", std::process::id()));
    let entry = root.join("project.veac");
    fs::create_dir_all(&root).unwrap();
    fs::write(&entry, SOURCE).unwrap();

    let (discovered, authored) = build_project_path(&entry).unwrap();
    assert_eq!(discovered, root.canonicalize().unwrap());
    assert_eq!(authored.manifest.targets.len(), 3);
    assert_eq!(
        build_project_root_path(&root, std::path::Path::new("project.veac"))
            .unwrap()
            .manifest
            .id
            .as_str(),
        "complete"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn authored_errors_and_entry_paths_have_stable_display_contracts() {
    let error = build_project_source(&SOURCE.replace("version: 1", "version: -1")).unwrap_err();
    assert!(error.to_string().contains("project value decode failed"));

    let authored = build_project_source(SOURCE).unwrap();
    assert_eq!(
        authored.manifest.targets[0]
            .entry
            .source_path()
            .unwrap()
            .as_str(),
        "base.veac"
    );
    assert_eq!(
        authored.manifest.targets[2]
            .entry
            .source_path()
            .unwrap()
            .as_str(),
        "evidence.veac"
    );
    assert!(
        build_project_source(DERIVATION).unwrap().manifest.targets[0]
            .entry
            .source_path()
            .is_none()
    );
}

#[test]
fn authoring_error_variants_preserve_their_public_context() {
    let load = build_project_path(std::path::Path::new("missing-project-entry.veac"))
        .unwrap_err()
        .to_string();
    assert!(load.contains("project source load failed"));

    let invalid = build_project_source(&SOURCE.replace("veac.project", "wrong.schema"))
        .unwrap_err()
        .to_string();
    assert!(invalid.contains("project contract has"));

    let language = build_project_source(
        "import \"./missing.veac\" as missing; fn workspace() -> ProjectManifest { missing.make() }",
    )
    .unwrap_err()
    .to_string();
    assert!(language.contains("project language failed"));

    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let serialization = ProjectAuthoringError::from(json).to_string();
    assert!(serialization.contains("project serialization failed"));
}
