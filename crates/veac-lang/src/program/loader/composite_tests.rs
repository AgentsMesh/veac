use super::CompositeSourceLoader;
use crate::package::source_id::{
    request as package_request, selector as parse_selector, source as package_source,
};
use crate::program::{FileSystemLoader, SourceAuthority, SourceLoader};

#[test]
fn package_selector_is_exact_and_namespaced() {
    let (identity, path) = parse_selector("widgets@1.2.3/card.veac").unwrap();
    assert_eq!(identity.name.as_str(), "widgets");
    assert_eq!(identity.version.as_str(), "1.2.3");
    assert_eq!(path, "card.veac");
    assert!(parse_selector("widgets@latest/card.veac").is_err());
}

#[test]
fn request_and_source_forms_are_unambiguous() {
    let (_, path) = package_request("package:widgets@1.2.3/card.veac")
        .unwrap()
        .unwrap();
    assert_eq!(path, "card.veac");
    let (_, internal) = package_source("packages/widgets@1.2.3/card.veac")
        .unwrap()
        .unwrap();
    assert_eq!(internal, "card.veac");
    assert!(package_request("card.veac").unwrap().is_none());
}

#[test]
fn malformed_qualified_paths_fail_closed() {
    for value in [
        "package:widgets/card.veac",
        "package:widgets@1.2.3",
        "package:../x@1.0.0/a.veac",
    ] {
        assert!(package_request(value).is_err());
    }
    assert!(package_source("packages/widgets@1.2.3/../secret.veac").is_err());
}

#[test]
fn package_source_namespace_rejects_empty_and_unsafe_segments() {
    for value in [
        "packages/widgets@1.2.3/",
        "packages/widgets@1.2.3/a//b.veac",
        "packages/widgets@1.2.3/a\\b.veac",
        "packages/widgets@1.2.3/a:secret.veac",
    ] {
        assert!(package_source(value).is_err(), "accepted {value}");
    }
}

#[test]
fn package_namespace_is_read_only_even_before_a_package_is_mounted() {
    let temp = tempfile::tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    std::fs::write(&entry, "module {}").unwrap();
    let (project, _) = FileSystemLoader::for_entry(&entry).unwrap();
    let loader = CompositeSourceLoader::new(project);

    assert_eq!(
        loader.authority("packages/widgets@1.2.3/card.veac"),
        SourceAuthority::ReadOnlyDependency
    );
    assert_eq!(loader.authority("main.veac"), SourceAuthority::Project);
}

#[test]
fn project_sources_cannot_claim_the_reserved_package_namespace() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("packages")).unwrap();
    let entry = temp.path().join("main.veac");
    std::fs::write(&entry, "module {}").unwrap();
    std::fs::write(temp.path().join("packages/fake.veac"), "module {}").unwrap();
    let (project, _) = FileSystemLoader::for_entry(&entry).unwrap();
    let loader = CompositeSourceLoader::new(project);

    let error = loader
        .load("main.veac", "./packages/fake.veac")
        .unwrap_err();
    assert!(error.contains("reserved package namespace"));
}
