use super::*;

#[test]
fn bound_child_removal_rejects_nonempty_directories() {
    let temp = tempfile::tempdir().unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let child = root.create_child("child").unwrap();
    child.write_all_sync("payload", b"content").unwrap();

    let error = root.remove_child("child", &child).unwrap_err();

    assert!(error.message.contains("remove transaction directory"));
    assert_eq!(
        std::fs::read(temp.path().join("child/payload")).unwrap(),
        b"content"
    );
}
