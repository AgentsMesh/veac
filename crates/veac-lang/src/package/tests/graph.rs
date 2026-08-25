use std::collections::BTreeMap;

use super::*;

fn index(values: &[LockedPackage]) -> BTreeMap<PackageIdentity, &LockedPackage> {
    values
        .iter()
        .map(|value| (value.package.clone(), value))
        .collect()
}

#[test]
fn graph_requires_exact_reachable_acyclic_lock_closure() {
    let values = vec![
        locked("alpha", "1.0.0", vec![dependency("beta", "1.0.0")]),
        locked("beta", "1.0.0", vec![]),
    ];
    assert!(graph::validate(&[dependency("alpha", "1.0.0")], &index(&values)).is_ok());
    let missing = graph::validate(&[dependency("alpha", "2.0.0")], &index(&values)).unwrap_err();
    assert_eq!(missing.kind(), PackageErrorKind::MissingLockedDependency);
    assert!(missing.message().contains("exact locked version"));
    assert!(graph::validate(&[dependency("beta", "1.0.0")], &index(&values)).is_err());

    let cycle = vec![
        locked("alpha", "1.0.0", vec![dependency("beta", "1.0.0")]),
        locked("beta", "1.0.0", vec![dependency("alpha", "1.0.0")]),
    ];
    assert!(
        graph::validate(&[dependency("alpha", "1.0.0")], &index(&cycle))
            .unwrap_err()
            .message()
            .contains("cycle")
    );
}
