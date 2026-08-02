use super::*;
use crate::emitter::{BackendCommand, BackendInput};
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

#[test]
fn command_input_outside_verified_bindings_is_rejected() {
    let plan = resolved(&fixture());
    let command = BackendCommand {
        preparations: Vec::new(),
        inputs: vec![BackendInput {
            path: "/tmp/unbound-media".into(),
        }],
        filter_graph: None,
        filter_contract: None,
        maps: vec![],
        output_args: vec![],
        output_path: "/tmp/out".into(),
    };
    let error = capabilities(&plan, &ExecutionBindings::default(), &command).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].code,
        "BACKEND_INPUT_CAPABILITY_UNRESOLVED"
    );
}

#[test]
fn absent_decoder_and_demuxer_facts_are_typed_capability_errors() {
    let mut plan = resolved(&fixture());
    let input = &mut plan.inputs[0];
    input.video = None;
    let decoder = source_decoder(input, MediaRole::Video).unwrap_err();
    assert_eq!(
        decoder.diagnostics()[0].code,
        "BACKEND_INPUT_CAPABILITY_UNRESOLVED"
    );

    input.probe = None;
    let demuxer = source_demuxer(input).unwrap_err();
    assert_eq!(
        demuxer.diagnostics()[0].code,
        "BACKEND_INPUT_CAPABILITY_UNRESOLVED"
    );
}
