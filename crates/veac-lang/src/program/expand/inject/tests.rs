use super::{ensure_working_set, rendered_len, Sequences};

#[test]
fn exact_working_set_boundary_includes_inputs_and_output() {
    let project = "project p {}".to_owned();
    let id = "a";
    let body = "layer visual v {}";
    let output_len = project.len() + rendered_len("test.veac", id, body).unwrap();
    let exact = project.len() + id.len() + body.len() + output_len;
    let mut sequences = Sequences::with_limit("test.veac", project.clone(), exact).unwrap();
    sequences.push(id, body.to_owned(), 0).unwrap();
    assert_eq!(sequences.finish().len(), output_len);

    let mut too_small = Sequences::with_limit("test.veac", project, exact - 1).unwrap();
    let error = too_small.push(id, body.to_owned(), 0).unwrap_err();
    assert_eq!(error.code, "PROGRAM_EXPANSION_BUDGET");
    assert!(too_small.values.is_empty());
}

#[test]
fn project_without_sequences_reuses_its_owned_buffer() {
    let project = "project p {}".to_owned();
    let pointer = project.as_ptr();
    let sequences = Sequences::with_limit("test.veac", project, 12).unwrap();
    let output = sequences.finish();
    assert_eq!(output.as_ptr(), pointer);
    assert!(Sequences::with_limit("test.veac", output, 11).is_err());
}

#[test]
fn active_component_frame_participates_in_final_allocation_budget() {
    let project = "project p {}".to_owned();
    let id = "added";
    let body = "layer visual picture {}";
    let output_len = project.len() + rendered_len("test.veac", id, body).unwrap();
    let exact = project.len() + id.len() + body.len() + output_len;
    let mut sequences = Sequences::with_limit("test.veac", project, exact).unwrap();
    assert!(sequences.push(id, body.to_owned(), 1).is_err());
    assert!(sequences.values.is_empty());
}

#[test]
fn successful_assembly_preserves_layout_and_trims_body_without_copying() {
    let project = "project p {\n  sequence original {}\n}".to_owned();
    let id = "added";
    let body = "\nlayer visual picture {}\n";
    let output_len = project.len() + rendered_len("test.veac", id, body.trim()).unwrap();
    let limit = project.len() + id.len() + body.len() + output_len;
    let mut sequences = Sequences::with_limit("test.veac", project, limit).unwrap();
    sequences.push(id, body.to_owned(), 0).unwrap();
    assert_eq!(
        sequences.finish(),
        "project p {\n  sequence original {}\n\n  sequence added {\n  layer visual picture {}\n  }\n}"
    );
}

#[test]
fn malformed_project_and_working_set_overflow_fail_closed() {
    let result = Sequences::new("test.veac", "project p {".to_owned());
    let error = match result {
        Ok(_) => panic!("project without a closing brace was accepted"),
        Err(error) => error,
    };
    assert_eq!(error.code, "PROGRAM_PROJECT_BLOCK");
    assert!(ensure_working_set("test.veac", usize::MAX, 1, 0, 0, usize::MAX).is_err());
}
