use std::cell::RefCell;

use crate::{source::copy_with, verify_source, ArtifactErrorKind};

#[test]
fn verified_copy_never_publishes_or_unlinks_a_replaced_random_stage() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"same-content").unwrap();
    let expected = verify_source(&source, None).unwrap().identity;
    let replacement = RefCell::new(None);

    let error = copy_with(&source, &destination, Some(&expected), || {
        let stage = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".veac-stage-")
            })
            .unwrap()
            .join("payload");
        std::fs::remove_file(&stage).unwrap();
        std::fs::write(&stage, b"same-content").unwrap();
        replacement.replace(Some(stage));
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(
        std::fs::read(replacement.into_inner().unwrap()).unwrap(),
        b"same-content"
    );
    assert!(!destination.exists());
}
