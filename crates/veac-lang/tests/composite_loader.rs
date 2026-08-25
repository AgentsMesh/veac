use std::path::Path;

use veac_lang::package::{ExactVersion, PackageIdentity, PackageName, PackageSourceLoader};
use veac_lang::program::{CompilerDatabase, CompositeSourceLoader, FileSystemLoader, SourceLoader};

#[path = "package_contract_helpers.rs"]
#[allow(dead_code)]
mod helpers;

fn components() -> (PackageIdentity, PackageSourceLoader) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/veac-components");
    let (loader, _) = PackageSourceLoader::for_root(&root).expect("standard package contract");
    let identity = PackageIdentity {
        name: PackageName::new("veac-components").unwrap(),
        version: ExactVersion::new("0.1.0").unwrap(),
    };
    (identity, loader)
}

fn project_loader(temp: &tempfile::TempDir) -> (FileSystemLoader, String) {
    std::fs::write(temp.path().join("main.veac"), "project { output = 1s }\n").unwrap();
    let (loader, entry) = FileSystemLoader::for_entry(&temp.path().join("main.veac")).unwrap();
    (loader, entry.id)
}

#[test]
fn project_and_verified_package_share_one_source_loader() {
    let temp = tempfile::tempdir().unwrap();
    let (project, entry) = project_loader(&temp);
    let (identity, package) = components();
    let mut loader = CompositeSourceLoader::new(project);
    loader.mount_package(identity, package).unwrap();

    let imported = loader
        .load(&entry, "package:veac-components@0.1.0/main.veac")
        .unwrap();
    assert_eq!(imported.id, "packages/veac-components@0.1.0/main.veac");
    let relative = loader.load(&imported.id, "./layout.veac").unwrap();
    assert_eq!(relative.id, "packages/veac-components@0.1.0/layout.veac");
    assert!(relative.source.contains("export"));
}

#[test]
fn package_entry_is_available_with_a_stable_namespace() {
    let (identity, package) = components();
    let temp = tempfile::tempdir().unwrap();
    let (project, _) = project_loader(&temp);
    let mut loader = CompositeSourceLoader::new(project);
    loader.mount_package(identity.clone(), package).unwrap();
    let entry = loader.load_package_entry(&identity).unwrap();
    assert_eq!(entry.id, "packages/veac-components@0.1.0/main.veac");
}

#[test]
fn mounting_rejects_an_identity_different_from_the_verified_package() {
    let (_, package) = components();
    let temp = tempfile::tempdir().unwrap();
    let (project, _) = project_loader(&temp);
    let mut loader = CompositeSourceLoader::new(project);
    let wrong = PackageIdentity {
        name: PackageName::new("other-package").unwrap(),
        version: ExactVersion::new("0.1.0").unwrap(),
    };
    let error = loader.mount_package(wrong, package).unwrap_err();
    assert!(error.contains("does not match loader identity"));
}

#[test]
fn qualified_dependency_import_uses_the_importer_package_registry() {
    let package_root = tempfile::tempdir().unwrap();
    helpers::write_contract(package_root.path());
    let (package_loader, entry) = PackageSourceLoader::for_root(package_root.path()).unwrap();
    let project_root = tempfile::tempdir().unwrap();
    let (project, _) = project_loader(&project_root);
    let mut loader = CompositeSourceLoader::new(project);
    let root = PackageIdentity {
        name: PackageName::new("root").unwrap(),
        version: ExactVersion::new("1.0.0").unwrap(),
    };
    let util = PackageIdentity {
        name: PackageName::new("util").unwrap(),
        version: ExactVersion::new("2.0.0").unwrap(),
    };
    loader.mount_package(root.clone(), package_loader).unwrap();
    let imported = loader
        .load(
            "packages/root@1.0.0/main.veac",
            "package:util@2.0.0/lib.veac",
        )
        .unwrap();
    assert_eq!(imported.id, "packages/util@2.0.0/lib.veac");
    assert_eq!(entry.id, "packages/root@1.0.0/main.veac");
    assert_eq!(
        loader.load_package_entry(&root).unwrap().id,
        "packages/root@1.0.0/main.veac"
    );
    assert!(loader.load_package_entry(&util).is_err());
}

#[test]
fn executable_project_calls_verified_component_package_to_canonical_ir() {
    let temp = tempfile::tempdir().unwrap();
    let source = r#"import "package:veac-components@0.1.0/main.veac" as components;

fn main(context: Context) -> Project {
  let card = components.card(
    key: identifier("offer"),
    at: 0s,
    title: "架构升级",
    detail: "可验证组件包",
  );
  let layer = visual_layer(
    identifier("cards"), 0, placement_free(),
    track_state(track_playback_enabled(), track_audio_audible(),
      track_isolation_normal(), track_editing_unlocked()),
    track_routing_default(),
  ).with_item(components.card_item(card));
  let timeline = sequence(
    identifier("main"), "组件包集成",
    sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000),
  ).with_layer(layer);
  let output = components.preview(timeline, canvas(1280px, 720px), frame_rate(30, 1));
  project(identifier("package-project"), project_settings(600))
    .with_sequence(timeline).entry(timeline).with_delivery(output)
}
"#;
    let entry_path = temp.path().join("main.veac");
    std::fs::write(&entry_path, source).unwrap();
    let (project, entry) = FileSystemLoader::for_entry(&entry_path).unwrap();
    let (identity, package) = components();
    let mut loader = CompositeSourceLoader::new(project);
    loader.mount_package(identity, package).unwrap();

    let built = CompilerDatabase::default()
        .build_with_loader(entry, &loader)
        .unwrap();
    let canonical = veac_ir::canonical_json(built.envelope()).unwrap();
    assert_eq!(
        built.envelope().project.sequences[0].tracks[0].clips.len(),
        1
    );
    assert_eq!(built.envelope().project.render_configs.len(), 1);
    assert_eq!(
        built.envelope().project.render_configs[0]
            .deliverables
            .len(),
        1
    );
    assert!(built
        .sources()
        .contains_key("packages/veac-components@0.1.0/main.veac"));
    assert_eq!(
        veac_ir::decode_canonical_json(&canonical).unwrap(),
        *built.envelope()
    );
}
