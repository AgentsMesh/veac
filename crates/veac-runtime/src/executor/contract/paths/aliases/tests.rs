use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::{detect, fold_name, validate_with, Policy};
use crate::executor::contract::paths::declaration::{self, Declaration};

#[test]
fn forced_case_and_unicode_policy_rejects_future_static_aliases() {
    let parent = PathBuf::from("/outputs");
    let policy = insensitive();
    let policies = BTreeMap::from([(parent.clone(), policy)]);
    let declarations = [
        Declaration::Static(parent.join("Caf\u{e9}.MOV")),
        Declaration::Static(parent.join("CAFE\u{301}.mov")),
    ];
    let error = validate_with(&declarations, &BTreeSet::new(), &policies).unwrap_err();
    assert!(error.message.contains("filesystem policy"));
}

#[test]
fn folded_patterns_and_protected_paths_use_the_same_policy() {
    let parent = PathBuf::from("/outputs");
    let policies = BTreeMap::from([(parent.clone(), insensitive())]);
    let patterns = [
        Declaration::Pattern {
            parent: parent.clone(),
            prefix: b"Frame-".to_vec(),
            suffix: b".PNG".to_vec(),
            minimum_width: None,
        },
        Declaration::Pattern {
            parent: parent.clone(),
            prefix: b"frame-".to_vec(),
            suffix: b".png".to_vec(),
            minimum_width: Some(4),
        },
    ];
    assert!(validate_with(&patterns, &BTreeSet::new(), &policies).is_err());

    let output = [Declaration::Static(parent.join("MASTER.mov"))];
    let protected = BTreeSet::from([parent.join("master.MOV")]);
    assert!(validate_with(&output, &protected, &policies).is_err());
}

#[test]
fn padded_pattern_matching_preserves_its_minimum_width() {
    let parent = PathBuf::from("/outputs");
    let pattern = Declaration::Pattern {
        parent: parent.clone(),
        prefix: b"frame-".to_vec(),
        suffix: b".png".to_vec(),
        minimum_width: Some(4),
    };
    let short = Declaration::Static(parent.join("frame-1.png"));
    let padded = Declaration::Static(parent.join("frame-0001.png"));
    assert!(!declaration::overlaps(&pattern, &short));
    assert!(declaration::overlaps(&pattern, &padded));
}

#[test]
fn management_names_are_reserved_and_sensitive_policy_preserves_distinct_names() {
    let parent = PathBuf::from("/outputs");
    let sensitive = Policy {
        case_insensitive: false,
        normalization_insensitive: false,
    };
    let policies = BTreeMap::from([(parent.clone(), sensitive)]);
    let distinct = [
        Declaration::Static(parent.join("Title.mov")),
        Declaration::Static(parent.join("title.mov")),
    ];
    validate_with(&distinct, &BTreeSet::new(), &policies).unwrap();
    let reserved = [Declaration::Passlog {
        parent,
        prefix: b".VEAC-pass".to_vec(),
    }];
    assert!(validate_with(&reserved, &BTreeSet::new(), &policies).is_err());
}

#[test]
fn policy_probe_and_name_folding_are_deterministic() {
    let temp = tempfile::tempdir().unwrap();
    let policy = detect(temp.path()).unwrap();
    assert_eq!(
        fold_name("Cafe\u{301}.MOV", &insensitive()),
        "caf\u{e9}.mov"
    );
    let unchanged = fold_name("Title.MOV", &policy);
    assert!(!unchanged.is_empty());
    assert!(!temp.path().join("VEAC-Case-A").exists());
}

fn insensitive() -> Policy {
    Policy {
        case_insensitive: true,
        normalization_insensitive: true,
    }
}
