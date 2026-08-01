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

#[test]
fn tree_overlap_is_symmetric_for_every_declaration_kind() {
    let root = tree("/outputs/render");
    for nested in [
        fixed("/outputs/render/final.mov"),
        tree("/outputs/render/segments"),
        pattern("/outputs/render/frames", b"frame-", b".png"),
        passlog("/outputs/render/pass", b"encode"),
    ] {
        assert_symmetric_overlap(&root, &nested, true);
    }
    for outside in [
        fixed("/outputs/preview.mov"),
        tree("/outputs/preview"),
        pattern("/outputs/preview", b"frame-", b".png"),
        passlog("/outputs/preview", b"encode"),
    ] {
        assert_symmetric_overlap(&root, &outside, false);
    }

    let numbered_tree = tree("/outputs/frame-42.png");
    assert_symmetric_overlap(
        &numbered_tree,
        &pattern("/outputs", b"frame-", b".png"),
        true,
    );
    let passlog_tree = tree("/outputs/encode-7.log.audit");
    assert_symmetric_overlap(&passlog_tree, &passlog("/outputs", b"encode"), true);
}

#[test]
fn passlog_overlap_is_symmetric_for_static_and_dynamic_outputs() {
    let logs = passlog("/outputs", b"encode");
    assert_symmetric_overlap(&logs, &fixed("/outputs/encode-12.log.audit"), true);
    assert_symmetric_overlap(&logs, &fixed("/outputs/encode.log"), false);
    assert_symmetric_overlap(&logs, &pattern("/outputs", b"encode-", b".log"), true);
    assert_symmetric_overlap(&logs, &pattern("/outputs", b"frame-", b".png"), false);
    assert_symmetric_overlap(&logs, &passlog("/outputs", b"encode-1.log.part"), true);
    assert_symmetric_overlap(&logs, &passlog("/outputs", b"audio"), false);
}

fn assert_symmetric_overlap(left: &Declaration, right: &Declaration, expected: bool) {
    assert_eq!(declaration::overlaps(left, right), expected);
    assert_eq!(declaration::overlaps(right, left), expected);
}

fn fixed(path: &str) -> Declaration {
    Declaration::Static(path.into())
}

fn tree(path: &str) -> Declaration {
    Declaration::Tree(path.into())
}

fn pattern(parent: &str, prefix: &[u8], suffix: &[u8]) -> Declaration {
    Declaration::Pattern {
        parent: parent.into(),
        prefix: prefix.to_vec(),
        suffix: suffix.to_vec(),
        minimum_width: None,
    }
}

fn passlog(parent: &str, prefix: &[u8]) -> Declaration {
    Declaration::Passlog {
        parent: parent.into(),
        prefix: prefix.to_vec(),
    }
}

fn insensitive() -> Policy {
    Policy {
        case_insensitive: true,
        normalization_insensitive: true,
    }
}
