use std::path::PathBuf;

use veac_codegen::emitter::BackendInput;

use super::*;
use crate::executor::tests::support::command;

#[test]
fn every_structured_input_gets_the_fail_closed_demux_policy() {
    let mut command = command(&PathBuf::from("output"));
    command.inputs = ["first.mp4", "second.wav"]
        .into_iter()
        .map(|path| BackendInput {
            path: PathBuf::from(path),
        })
        .collect();

    let arguments = for_command(&command);
    let inputs = arguments
        .windows(6)
        .filter(|values| {
            values[0] == "-protocol_whitelist"
                && values[1] == "file,pipe"
                && values[2] == "-format_whitelist"
                && values[4] == "-i"
        })
        .collect::<Vec<_>>();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0][5], "first.mp4");
    assert_eq!(inputs[1][5], "second.wav");
    assert!(!inputs[0][3].split(',').any(|value| value == "hls"));
}

#[test]
fn commands_without_inputs_keep_their_canonical_arguments() {
    let command = command(&PathBuf::from("output"));
    assert_eq!(for_command(&command), command.to_args());
}

#[test]
fn script_arguments_keep_input_policy_and_replace_only_the_filter_transport() {
    let mut command = command(&PathBuf::from("output"));
    command.inputs.push(BackendInput {
        path: PathBuf::from("input.mp4"),
    });
    command.filter_graph = Some("null[outv]".to_owned());
    let path = PathBuf::from("stage/filter-complex.ffscript");
    let arguments = for_filter_script(&command, &path);
    assert!(arguments
        .windows(2)
        .any(|pair| pair[0] == "-filter_complex_threads" && pair[1] == "1"));
    assert!(arguments
        .windows(2)
        .any(|pair| pair[0] == "-threads" && pair[1] == "1"));
    assert_eq!(
        arguments
            .windows(2)
            .filter(|pair| pair[0] == "-threads" && pair[1] == "1")
            .count(),
        command.inputs.len() + 1
    );
    assert_eq!(
        &arguments[arguments.len() - 3..],
        ["-threads", "1", command.output_path.to_str().unwrap()]
    );
    assert!(arguments
        .windows(2)
        .any(|pair| { pair[0] == "-filter_complex_script" && pair[1] == path.to_string_lossy() }));
    assert!(!arguments.iter().any(|value| value == "-filter_complex"));
    assert!(arguments
        .windows(2)
        .any(|pair| pair[0] == "-i" && pair[1] == "input.mp4"));
    assert!(arguments.iter().any(|value| value == "-protocol_whitelist"));
}

#[test]
fn alpha_prores_inline_graph_keeps_thread_limits_in_their_scopes() {
    let mut command = command(&PathBuf::from("output.mov"));
    command.inputs.push(BackendInput {
        path: PathBuf::from("input.mov"),
    });
    command.filter_graph = Some("null[outv]".to_owned());
    command.output_args = vec![
        "-c:v".into(),
        "prores_ks".into(),
        "-pix_fmt".into(),
        "yuva444p12le".into(),
    ];
    let arguments = for_command(&command);
    assert!(arguments
        .windows(2)
        .any(|pair| pair[0] == "-filter_complex_threads" && pair[1] == "1"));
    assert_eq!(
        arguments
            .windows(2)
            .filter(|pair| pair[0] == "-threads" && pair[1] == "1")
            .count(),
        command.inputs.len() + 1
    );
    assert_eq!(
        &arguments[arguments.len() - 3..],
        ["-threads", "1", "output.mov"]
    );
}
