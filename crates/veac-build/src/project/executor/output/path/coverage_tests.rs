use super::{failed, fingerprint};
use crate::CancellationToken;

#[test]
fn owned_failures_and_directory_reads_are_closed() {
    assert_eq!(failed(String::from("owned")).message(), "owned");
    let temp = tempfile::tempdir().unwrap();
    let error = fingerprint(temp.path(), &CancellationToken::new()).unwrap_err();
    assert!(error.message().contains("cannot read backend output"));
}
