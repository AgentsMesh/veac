use std::path::Path;

use veac_artifact::ContentDigest;
use veac_build::{ProjectFileSnapshot, ProjectSourceGraphRevision};

use super::{prepare, verify, verify_authored};

const ENTRY: &str = r#"import "./helper.veac" as helper;
fn evidence() -> EvidenceSuite {
  EvidenceSuite {
    schema_version: 1,
    id: identifier(helper.suite_name()),
    sources: [
      EvidenceSource {
        id: identifier("rendered"),
        binding: EvidenceSourceBinding.BoundInput { input_id: identifier("source"), },
      }
    ],
    samples: [],
    regions: [],
    assertions: [
      EvidenceAssertion.DecodeComplete {
        id: identifier("decode"),
        source_id: identifier("rendered"),
        minimum_frames: 1,
      }
    ],
  }
}
"#;

#[test]
fn imported_module_change_after_snapshot_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("evidence.veac"), ENTRY).unwrap();
    write_helper(temp.path(), "suite");
    let (entry, revision) = capture(temp.path());

    prepare(temp.path(), &entry, &revision).unwrap();
    write_helper(temp.path(), "changed-suite");
    assert_eq!(ContentDigest::sha256(ENTRY.as_bytes()), entry.content);

    let error = verify(temp.path(), &entry, &revision).unwrap_err();
    assert!(error.message().contains("source graph revision changed"));
}

#[test]
fn verification_rejects_each_closed_revision_mismatch() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("evidence.veac"), ENTRY).unwrap();
    write_helper(temp.path(), "suite");
    let authored = authored(temp.path());
    let (entry, expected) = captured(&authored);

    let mut wrong_root = expected.clone();
    wrong_root.root_module = "other.veac".to_owned();
    assert_error(
        verify_authored(&authored, &entry, &wrong_root),
        "root module changed",
    );

    let mut omitted = authored.clone();
    omitted.root_module = "missing.veac".to_owned();
    wrong_root.root_module = omitted.root_module.clone();
    assert_error(
        verify_authored(&omitted, &entry, &wrong_root),
        "omitted its root module",
    );

    let mut wrong_entry = entry.clone();
    wrong_entry.size_bytes += 1;
    assert_error(
        verify_authored(&authored, &wrong_entry, &expected),
        "source entry changed",
    );

    let mut wrong_count = expected.clone();
    wrong_count.module_count += 1;
    assert_error(
        verify_authored(&authored, &entry, &wrong_count),
        "graph revision changed",
    );
    let mut wrong_modules = expected.clone();
    wrong_modules.modules.pop();
    assert_error(
        verify_authored(&authored, &entry, &wrong_modules),
        "graph revision changed",
    );
}

fn capture(root: &Path) -> (ProjectFileSnapshot, ProjectSourceGraphRevision) {
    captured(&authored(root))
}

fn authored(root: &Path) -> veac_evidence::AuthoredEvidenceSuite {
    veac_evidence::build_evidence_root_path(root, Path::new("evidence.veac")).unwrap()
}

fn captured(
    authored: &veac_evidence::AuthoredEvidenceSuite,
) -> (ProjectFileSnapshot, ProjectSourceGraphRevision) {
    let source = &authored.sources[&authored.root_module];
    (
        ProjectFileSnapshot {
            path: "evidence.veac".to_owned(),
            content: ContentDigest::sha256(source.as_bytes()),
            size_bytes: source.len() as u64,
        },
        ProjectSourceGraphRevision {
            root_module: authored.root_module.clone(),
            source_graph_sha256: authored.source_revision.source_graph_sha256.clone(),
            module_count: authored.sources.len() as u32,
            modules: authored.sources.keys().cloned().collect(),
        },
    )
}

fn assert_error(result: Result<(), veac_build::ProjectBackendError>, expected: &str) {
    let error = result.unwrap_err();
    assert!(error.message().contains(expected), "{error}");
}

fn write_helper(root: &Path, name: &str) {
    std::fs::write(
        root.join("helper.veac"),
        format!("module {{ export fn suite_name() -> text {{ \"{name}\" }} }}\n"),
    )
    .unwrap();
}
