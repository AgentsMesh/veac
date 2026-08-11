use super::{artifact_error, corrupt, io};

#[test]
fn helper_instantiations_preserve_unpack_failure_context() {
    assert_eq!(corrupt("literal").message(), "literal");
    assert_eq!(corrupt(String::from("owned")).message(), "owned");
    assert!(io(std::io::Error::other("disk"))
        .message()
        .contains("directory delivery I/O failed"));
    let artifact = artifact_error(std::io::Error::other("store").into());
    assert!(artifact
        .message()
        .contains("directory artifact delivery failure"));
}
