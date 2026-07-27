use std::path::Path;

use veac_codegen::emitter::MAX_INLINE_FILTER_GRAPH_BYTES;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn canonical_arguments_keep_small_filter_graphs_inline() {
    let plan = resolved(&fixture());
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    let graph = command.filter_graph.as_ref().unwrap();
    assert!(graph.len() <= MAX_INLINE_FILTER_GRAPH_BYTES);
    let arguments = command.to_args();
    let index = arguments
        .iter()
        .position(|value| value == "-filter_complex")
        .unwrap();
    assert_eq!(&arguments[index + 1], graph);
    assert!(!arguments
        .iter()
        .any(|value| value == "-filter_complex_script"));
}

#[test]
fn script_arguments_reference_the_typed_runtime_path_without_inlining() {
    let plan = resolved(&fixture());
    let mut command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    command.filter_graph = Some("x".repeat(MAX_INLINE_FILTER_GRAPH_BYTES + 1));
    let path = Path::new("/private/stage/filter-complex.ffscript");
    let arguments = command.to_args_with_filter_script(path);
    let index = arguments
        .iter()
        .position(|value| value == "-filter_complex_script")
        .unwrap();
    assert_eq!(arguments[index + 1], path.to_string_lossy());
    assert!(!arguments.iter().any(|value| value == "-filter_complex"));
    assert!(!arguments
        .iter()
        .any(|value| command.filter_graph.as_ref() == Some(value)));
}

#[test]
fn a_script_request_without_a_graph_emits_no_filter_option() {
    let plan = resolved(&fixture());
    let mut command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    command.filter_graph = None;
    command.filter_contract = None;
    let arguments = command.to_args_with_filter_script(Path::new("unused"));
    assert!(!arguments
        .iter()
        .any(|value| value == "-filter_complex" || value == "-filter_complex_script"));
}
