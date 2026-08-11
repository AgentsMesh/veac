use super::{failed, io};

#[test]
fn helper_instantiations_preserve_pack_failure_context() {
    assert_eq!(failed("literal").message(), "literal");
    assert_eq!(failed(String::from("owned")).message(), "owned");
    assert!(io(std::io::Error::other("disk"))
        .message()
        .contains("directory archive I/O failed"));
}
