use std::collections::BTreeMap;

use veac_artifact::ContentDigest;
use veac_project::ProjectPath;

use super::{capture_once, CapturedGraph};
use crate::{ProjectFileSnapshot, ProjectSourceGraphRevision};

#[test]
fn repeated_profile_source_paths_are_captured_once() {
    let path = ProjectPath::new("evidence.veac");
    let expected = captured();
    let mut cache = BTreeMap::new();
    let mut captures = 0;

    let first = capture_once(&mut cache, &path, || {
        captures += 1;
        Ok(expected.clone())
    })
    .unwrap();
    let second = capture_once(&mut cache, &path, || {
        captures += 1;
        Ok(expected.clone())
    })
    .unwrap();

    assert_eq!(captures, 1);
    assert_eq!(first, expected);
    assert_eq!(second, expected);
}

fn captured() -> CapturedGraph {
    (
        ProjectFileSnapshot {
            path: "evidence.veac".to_owned(),
            content: ContentDigest::sha256(b"entry"),
            size_bytes: 5,
        },
        ProjectSourceGraphRevision {
            root_module: "evidence.veac".to_owned(),
            source_graph_sha256: "1".repeat(64),
            module_count: 2,
            modules: vec!["evidence.veac".to_owned(), "helper.veac".to_owned()],
        },
    )
}
