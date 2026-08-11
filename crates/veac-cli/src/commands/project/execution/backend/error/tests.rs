use super::{ProjectOptionExt, ProjectResultExt};

#[test]
fn result_context_preserves_values_and_names_errors() {
    assert_eq!(Ok::<_, &str>(7).project_context("phase").unwrap(), 7);
    let error = Err::<(), _>("failure")
        .project_context("phase")
        .unwrap_err();
    assert_eq!(error.to_string(), "phase: failure");
}

#[test]
fn required_option_preserves_values_and_rejects_absence() {
    assert_eq!(Some(7).project_required("missing").unwrap(), 7);
    assert_eq!(
        None::<u8>
            .project_required("missing")
            .unwrap_err()
            .to_string(),
        "missing"
    );
}
