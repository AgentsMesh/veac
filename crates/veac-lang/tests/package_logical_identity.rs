use veac_lang::package::api::{from_module_interface, ApiExport};
use veac_lang::package::{PackageSourceLoader, VerifiedPackageMountSet};
use veac_lang::program::{CompilerDatabase, FileSystemLoader, SourceAuthority, SourceLoader};

#[path = "package_logical_identity/support.rs"]
mod support;

#[test]
fn exact_package_identity_is_independent_of_root_and_vendor_locator() {
    let alpha = tempfile::tempdir().unwrap();
    let beta = tempfile::tempdir().unwrap();
    let standalone = tempfile::tempdir().unwrap();
    support::write_host(alpha.path(), "alpha", "vendor/util");
    support::write_host(beta.path(), "beta", "third_party/cache/u");
    support::write_util_root(standalone.path());

    let (alpha_loader, _) = PackageSourceLoader::for_root(alpha.path()).unwrap();
    let (beta_loader, _) = PackageSourceLoader::for_root(beta.path()).unwrap();
    let (util_loader, util_entry) = PackageSourceLoader::for_root(standalone.path()).unwrap();
    let util = support::identity("util", "2.0.0");
    let alpha_source = alpha_loader.load_package(&util, "lib.veac").unwrap();
    let beta_source = beta_loader.load_package(&util, "lib.veac").unwrap();

    assert_eq!(alpha_source.id, support::UTIL_ID);
    assert_eq!(beta_source.id, support::UTIL_ID);
    assert_eq!(util_entry.id, support::UTIL_ID);
    let database = CompilerDatabase::default();
    let alpha_interface = database
        .module_interface(alpha_source, &alpha_loader)
        .unwrap();
    let beta_interface = database
        .module_interface(beta_source, &beta_loader)
        .unwrap();
    let util_interface = database.module_interface(util_entry, &util_loader).unwrap();
    assert_eq!(alpha_interface, beta_interface);
    assert_eq!(beta_interface, util_interface);
    assert_eq!(
        util_interface.methods[0].receiver.source_id,
        support::UTIL_ID
    );

    let alpha_api = from_module_interface(util.clone(), &alpha_interface);
    let beta_api = from_module_interface(util.clone(), &beta_interface);
    let util_api = from_module_interface(util, &util_interface);
    assert_eq!(alpha_api, beta_api);
    assert_eq!(beta_api, util_api);
    let receiver = util_api
        .exports
        .iter()
        .find_map(|export| match export {
            ApiExport::Method { receiver, .. } => Some(receiver),
            _ => None,
        })
        .expect("canonical API contains the exported method");
    assert_eq!(receiver.module, support::UTIL_ID);
}

#[test]
fn shared_dependency_has_one_canonical_source_graph_node() {
    let alpha = tempfile::tempdir().unwrap();
    let beta = tempfile::tempdir().unwrap();
    support::write_host(alpha.path(), "alpha", "vendor/util");
    support::write_host(beta.path(), "beta", "third_party/cache/u");
    let roots = vec![alpha.path().to_path_buf(), beta.path().to_path_buf()];
    let mounts = VerifiedPackageMountSet::capture(&roots).unwrap();

    let project = tempfile::tempdir().unwrap();
    let entry_path = project.path().join("main.veac");
    std::fs::write(
        &entry_path,
        r#"import "package:alpha@1.0.0/main.veac" as alpha;
import "package:beta@1.0.0/main.veac" as beta;
fn main(context: Context) -> Project {
  let timeline = sequence(
    identifier("main"), "逻辑模块身份",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000),
  );
  project(identifier("identity"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#,
    )
    .unwrap();
    let (project_loader, entry) = FileSystemLoader::for_entry(&entry_path).unwrap();
    let loader = mounts.loader(project_loader).unwrap();
    let built = CompilerDatabase::default()
        .build_with_loader(entry, &loader)
        .unwrap();

    assert_eq!(
        built
            .sources()
            .keys()
            .filter(|id| id.as_str() == support::UTIL_ID)
            .count(),
        1
    );
    assert!(built
        .sources()
        .contains_key("packages/alpha@1.0.0/main.veac"));
    assert!(built
        .sources()
        .contains_key("packages/beta@1.0.0/main.veac"));
    assert!(!built.sources().keys().any(|id| id.contains("vendor/util")));
    assert!(!built
        .sources()
        .keys()
        .any(|id| id.contains("third_party/cache")));
    assert_eq!(
        loader.authority(support::UTIL_ID),
        SourceAuthority::ReadOnlyDependency
    );
}

#[test]
fn direct_mounts_are_order_independent_and_reject_split_brain_closures() {
    let alpha = tempfile::tempdir().unwrap();
    let beta = tempfile::tempdir().unwrap();
    support::write_host(alpha.path(), "alpha", "vendor/util");
    support::write_host(beta.path(), "beta", "third_party/cache/u");
    let project = tempfile::tempdir().unwrap();
    let entry_path = project.path().join("main.veac");
    std::fs::write(&entry_path, "project { output = 1s }\n").unwrap();

    let (first_project, _) = FileSystemLoader::for_entry(&entry_path).unwrap();
    let (alpha_loader, _) = PackageSourceLoader::for_root(alpha.path()).unwrap();
    let (beta_loader, _) = PackageSourceLoader::for_root(beta.path()).unwrap();
    let mut first = veac_lang::program::CompositeSourceLoader::new(first_project);
    first
        .mount_package(support::identity("beta", "1.0.0"), beta_loader)
        .unwrap();
    first
        .mount_package(support::identity("alpha", "1.0.0"), alpha_loader)
        .unwrap();

    let (second_project, _) = FileSystemLoader::for_entry(&entry_path).unwrap();
    let (alpha_loader, _) = PackageSourceLoader::for_root(alpha.path()).unwrap();
    let (beta_loader, _) = PackageSourceLoader::for_root(beta.path()).unwrap();
    let mut second = veac_lang::program::CompositeSourceLoader::new(second_project);
    second
        .mount_package(support::identity("alpha", "1.0.0"), alpha_loader)
        .unwrap();
    second
        .mount_package(support::identity("beta", "1.0.0"), beta_loader)
        .unwrap();
    assert_eq!(
        first.load(support::UTIL_ID, "./lib.veac").unwrap(),
        second.load(support::UTIL_ID, "./lib.veac").unwrap()
    );

    let divergent = tempfile::tempdir().unwrap();
    let changed = format!("// distinct locked bytes\n{}", support::UTIL_SOURCE);
    support::write_host_with_util(divergent.path(), "gamma", "cache/util", &changed);
    let (divergent_loader, _) = PackageSourceLoader::for_root(divergent.path()).unwrap();
    let error = second
        .mount_package(support::identity("gamma", "1.0.0"), divergent_loader)
        .unwrap_err();
    assert!(error.contains("util@2.0.0"), "{error}");
    assert!(error.contains("conflicting portable closure"), "{error}");
}
