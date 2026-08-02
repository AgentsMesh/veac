use super::*;

#[test]
fn render_boundary_preserves_runtime_resource_classification() {
    let limited = runtime_error(RuntimeError::resource_limit("deadline"));
    assert!(limited.is_resource_limit());
    assert!(limited
        .to_string()
        .contains("error[RENDER_FAILED]: deadline"));

    let general = runtime_error(RuntimeError::new("failure"));
    assert!(!general.is_resource_limit());
    assert!(general
        .to_string()
        .contains("error[RENDER_FAILED]: failure"));
}
