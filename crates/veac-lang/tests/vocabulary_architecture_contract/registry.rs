use std::{collections::BTreeSet, fs};

use veac_lang::vocabulary::{language_spec, ControlWord, VocabularyCategory};

use super::support::{manifest_dir, read, read_relative};

#[test]
fn contextual_catalog_and_control_groups_are_bidirectional() {
    let vocabulary = language_spec().vocabulary;
    let registered = ControlWord::all()
        .into_iter()
        .map(ControlWord::as_str)
        .collect::<BTreeSet<_>>();
    let published = vocabulary
        .in_category(VocabularyCategory::ContextualControl)
        .map(|entry| entry.spelling.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(published, registered);
    for spelling in published {
        assert_eq!(ControlWord::parse(spelling).unwrap().as_str(), spelling);
    }
}

#[test]
fn every_control_module_participates_in_the_group_registry() {
    let directory = manifest_dir().join("src/vocabulary/control/uses");
    let mut modules = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|value| value == "rs"))
        .filter(|path| !path.ends_with("mod.rs"))
        .collect::<Vec<_>>();
    modules.sort();
    let registry = read_relative("src/vocabulary/control/uses/mod.rs");
    assert_eq!(registry.matches("::ALL").count(), modules.len());
    for path in modules {
        let module = path.file_stem().unwrap().to_str().unwrap();
        assert!(
            registry.contains(&format!("pub(crate) mod {module};")),
            "{module} is not declared"
        );
        assert_eq!(
            registry.matches(&format!("{module}::ALL")).count(),
            1,
            "{module} must occur in GROUPS exactly once"
        );
        let source = read(&path);
        assert_eq!(source.matches("define_control_uses!").count(), 1);
        assert!(!source.contains("ControlUse::new"), "{}", path.display());
    }
}

#[test]
fn catalog_derives_exact_owner_metadata_from_control_groups() {
    let source = read_relative("src/vocabulary/catalog.rs");
    for expression in [
        "for usage in all_control_uses()",
        "usage.as_str()",
        "usage.position()",
        "usage.role()",
    ] {
        assert!(
            source.contains(expression),
            "catalog is missing {expression}"
        );
    }
    assert_eq!(
        source
            .matches("VocabularyCategory::ContextualControl")
            .count(),
        1,
        "contextual controls need one catalog publication path"
    );
}
