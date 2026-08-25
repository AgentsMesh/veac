use super::*;

#[path = "security/locks.rs"]
mod locks;

#[test]
fn source_edit_rejects_structural_injection_without_writing() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("injection.json");
    let edit = batch(
        revision(&source),
        "2s; const text injected = \"unexpected\"",
    );
    std::fs::write(&batch_path, serde_json::to_string_pretty(&edit).unwrap()).unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid expression"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}

#[test]
fn source_edit_cannot_overwrite_its_source_graph_lock() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("source-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
            revision(&source),
            "400ms",
        ))
        .unwrap(),
    )
    .unwrap();
    let lock = temp.path().join(".veac-source.lock");

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
            lock.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
    assert!(lock.is_file());
}

#[test]
fn source_edit_rejects_a_concurrent_source_graph_transaction() {
    use rustix::fs::{flock, open, openat, FlockOperation, Mode, OFlags};

    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("locked.json");
    let edit = batch(revision(&source), "400ms");
    std::fs::write(&batch_path, serde_json::to_string_pretty(&edit).unwrap()).unwrap();
    let directory = open(
        temp.path(),
        OFlags::RDONLY | OFlags::DIRECTORY,
        Mode::empty(),
    )
    .unwrap();
    let lock = openat(
        directory,
        ".veac-source.lock",
        OFlags::CREATE | OFlags::RDWR,
        Mode::RUSR | Mode::WUSR,
    )
    .unwrap();
    flock(&lock, FlockOperation::NonBlockingLockExclusive).unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOURCE_LOCKED"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}

#[test]
fn source_edit_rejects_a_distinct_hard_link_output() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let output = temp.path().join("hard-link.veac");
    std::fs::hard_link(&source, &output).unwrap();
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("hard-link.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
            revision(&source),
            "700ms",
        ))
        .unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));

    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
    assert_eq!(std::fs::read_to_string(output).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn source_commands_reject_a_final_symlink_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let target = outside.path().join("outside.veac");
    std::fs::write(&target, program_source()).unwrap();
    let source = temp.path().join("main.veac");
    symlink(&target, &source).unwrap();
    let original = std::fs::read_to_string(&target).unwrap();

    for command in ["check", "build", "source-revision", "fmt"] {
        veac()
            .args([command, source.to_str().unwrap()])
            .assert()
            .failure();
    }

    let revision = veac_lang::source_edit::SourceRevision {
        authored_source_graph_sha256: "0".repeat(64),
        complete_source_graph_sha256: "0".repeat(64),
    };
    let batch_path = temp.path().join("symlink.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(revision, "700ms"))
            .unwrap(),
    )
    .unwrap();
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure();

    assert_eq!(std::fs::read_to_string(target).unwrap(), original);
}
