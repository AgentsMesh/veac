use crate::*;

#[test]
fn relink_discovery_hashes_a_deterministic_recursive_file_set() {
    let temp = tempfile::tempdir().unwrap();
    let nested = temp.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(temp.path().join("b.mov"), b"b").unwrap();
    std::fs::write(nested.join("a.mov"), b"a").unwrap();
    let roots = [temp.path().to_owned(), temp.path().to_owned()];
    let candidates = discover_relink_candidates(&roots).unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0].path < candidates[1].path);
    assert_eq!(
        candidates[0].identity.digest,
        ContentDigest::sha256(std::fs::read(&candidates[0].path).unwrap()).value
    );
    assert_eq!(
        discover_relink_candidates(&[]).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn relink_depth_and_entry_budgets_accept_the_boundary_then_fail_closed() {
    let deep = tempfile::tempdir().unwrap();
    let first = deep.path().join("first");
    std::fs::create_dir(&first).unwrap();
    std::fs::write(first.join("media"), b"x").unwrap();
    let depth = limits(1, 10, 1, 1);
    assert_eq!(discover(deep.path(), depth).unwrap().len(), 1);
    let second = first.join("second");
    std::fs::create_dir(&second).unwrap();
    std::fs::write(second.join("other"), b"y").unwrap();
    assert_limit(discover(deep.path(), depth));

    let wide = tempfile::tempdir().unwrap();
    std::fs::write(wide.path().join("a"), b"a").unwrap();
    std::fs::write(wide.path().join("b"), b"b").unwrap();
    let entries = limits(0, 3, 1, 2);
    assert_eq!(discover(wide.path(), entries).unwrap().len(), 2);
    std::fs::write(wide.path().join("c"), b"c").unwrap();
    assert_limit(discover(wide.path(), entries));
}

#[test]
fn relink_file_and_aggregate_byte_budgets_are_independent() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("a"), b"abc").unwrap();
    std::fs::write(temp.path().join("b"), b"def").unwrap();

    assert_limit(discover(temp.path(), limits(0, 3, 2, 6)));
    assert_limit(discover(temp.path(), limits(0, 3, 3, 5)));
    assert_eq!(discover(temp.path(), limits(0, 3, 3, 6)).unwrap().len(), 2);
}

#[test]
fn relink_limits_may_tighten_but_never_disable_or_raise_hard_caps() {
    let hard = RelinkDiscoveryLimits::default();
    let invalid = [
        RelinkDiscoveryLimits {
            max_depth: hard.max_depth + 1,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_entries: 0,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_entries: hard.max_entries + 1,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_file_bytes: 0,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_file_bytes: hard.max_file_bytes + 1,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_total_bytes: 0,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_total_bytes: hard.max_total_bytes + 1,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_wall_time: std::time::Duration::ZERO,
            ..hard
        },
        RelinkDiscoveryLimits {
            max_wall_time: hard.max_wall_time + std::time::Duration::from_secs(1),
            ..hard
        },
    ];
    for limits in invalid {
        let error = discover_relink_candidates_with_limits(&[".".into()], limits).unwrap_err();
        assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
    }
}

#[cfg(unix)]
#[test]
fn relink_discovery_rejects_symlink_roots_entries_and_cycles() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("media"), b"media").unwrap();
    let linked = temp.path().join("linked");
    symlink(&target, &linked).unwrap();
    assert_unsafe(discover_relink_candidates(&[linked]));

    let root = tempfile::tempdir().unwrap();
    symlink(target.join("media"), root.path().join("media")).unwrap();
    assert_unsafe(discover_relink_candidates(&[root.path().to_owned()]));

    let cycle = tempfile::tempdir().unwrap();
    let nested = cycle.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    symlink(cycle.path(), nested.join("cycle")).unwrap();
    assert_unsafe(discover_relink_candidates(&[cycle.path().to_owned()]));
}

#[cfg(unix)]
#[test]
fn relink_discovery_rejects_special_files() {
    let temp = tempfile::tempdir().unwrap();
    let fifo = temp.path().join("fifo");
    assert!(std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success());
    assert_unsafe(discover_relink_candidates(&[temp.path().to_owned()]));
}

#[test]
fn relink_discovery_rejects_non_directory_roots() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("media");
    std::fs::write(&file, b"media").unwrap();
    assert_unsafe(discover_relink_candidates(&[file]));
}

fn discover(
    path: &std::path::Path,
    limits: RelinkDiscoveryLimits,
) -> ArtifactResult<Vec<RelinkCandidate>> {
    discover_relink_candidates_with_limits(&[path.to_owned()], limits)
}

fn limits(depth: usize, entries: usize, file: u64, total: u64) -> RelinkDiscoveryLimits {
    RelinkDiscoveryLimits {
        max_depth: depth,
        max_entries: entries,
        max_file_bytes: file,
        max_total_bytes: total,
        max_wall_time: MAX_RELINK_DISCOVERY_WALL_TIME,
    }
}

fn assert_limit(result: ArtifactResult<Vec<RelinkCandidate>>) {
    assert_eq!(result.unwrap_err().kind, ArtifactErrorKind::ResourceLimit);
}

fn assert_unsafe(result: ArtifactResult<Vec<RelinkCandidate>>) {
    assert_eq!(result.unwrap_err().kind, ArtifactErrorKind::UnsafePath);
}
