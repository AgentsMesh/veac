use super::*;

#[test]
fn registry_error_exposes_the_same_message_through_all_public_views() {
    let error = MethodRegistryError::new("METHOD_TEST", String::from("invalid method"));
    assert_eq!(error.code(), "METHOD_TEST");
    assert_eq!(error.message(), "invalid method");
    assert_eq!(error.to_string(), "invalid method");
}
