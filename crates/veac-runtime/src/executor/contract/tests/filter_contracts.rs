use std::path::PathBuf;

use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendFilterBinding, BackendFilterContract,
    BackendFilterEscape, BackendInput, BackendOutput, BackendProduct, MAX_FILTER_GRAPH_BYTES,
};

use super::super::filter;
use super::support::bundle;

const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";

#[test]
fn rendered_graph_must_match_its_typed_contract() {
    let temp = tempfile::tempdir().unwrap();
    let resource = temp.path().join("lut.cube");
    std::fs::write(&resource, b"lut").unwrap();
    let mut value = ffmpeg_bundle(&temp.path().join("output.bin"), &resource);
    let command = command_mut(&mut value);
    command.filter_contract = Some(file_contract(&resource));
    command.filter_graph = Some("different".to_owned());
    assert!(filter::validate(&value)
        .unwrap_err()
        .message
        .contains("does not match"));
}

#[test]
fn filter_resources_must_be_protected_and_unique() {
    let temp = tempfile::tempdir().unwrap();
    let resource = temp.path().join("font.ttf");
    std::fs::write(&resource, b"font").unwrap();
    let mut value = ffmpeg_bundle(&temp.path().join("output.bin"), &resource);
    let duplicate = BackendFilterBinding::directory(
        TOKEN.to_owned(),
        temp.path().to_path_buf(),
        vec![resource.clone(), resource.clone()],
        BackendFilterEscape::Quoted,
    );
    let contract = BackendFilterContract::new(TOKEN.to_owned(), vec![duplicate]).unwrap();
    let command = command_mut(&mut value);
    command.filter_graph = Some(contract.render_original().unwrap());
    command.filter_contract = Some(contract);
    assert!(filter::validate(&value)
        .unwrap_err()
        .message
        .contains("duplicate file"));

    value.protected_resources.clear();
    let command = command_mut(&mut value);
    command.filter_contract = Some(file_contract(&resource));
    command.filter_graph = command
        .filter_contract
        .as_ref()
        .map(|contract| contract.render_original().unwrap());
    assert!(filter::validate(&value)
        .unwrap_err()
        .message
        .contains("protected bundle resource"));
}

#[test]
fn missing_contract_sides_and_oversized_graphs_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let resource = temp.path().join("lut.cube");
    std::fs::write(&resource, b"lut").unwrap();
    let mut value = ffmpeg_bundle(&temp.path().join("output.bin"), &resource);
    command_mut(&mut value).filter_graph = Some(TOKEN.to_owned());
    assert!(filter::validate(&value).is_err());

    let command = command_mut(&mut value);
    command.filter_graph = None;
    command.filter_contract = Some(file_contract(&resource));
    assert!(filter::validate(&value).is_err());

    let command = command_mut(&mut value);
    command.filter_contract = None;
    command.filter_graph = Some("x".repeat(MAX_FILTER_GRAPH_BYTES + 1));
    assert!(filter::validate(&value)
        .unwrap_err()
        .message
        .contains("graph limit"));
}

fn ffmpeg_bundle(
    output: &std::path::Path,
    resource: &std::path::Path,
) -> crate::executor::model::RuntimeBundle {
    let mut value = bundle(output);
    value.tasks[0].product = BackendProduct::VideoMaster;
    value.tasks[0].action = BackendAction::Ffmpeg(BackendCommand {
        inputs: vec![BackendInput {
            path: resource.to_path_buf(),
        }],
        filter_graph: None,
        filter_contract: None,
        maps: Vec::new(),
        output_args: Vec::new(),
        output_path: output.to_path_buf(),
    });
    value.tasks[0].output = BackendOutput::File(output.to_path_buf());
    value
        .protected_resources
        .push(veac_codegen::emitter::BackendResource {
            path: resource.to_path_buf(),
            expected_identity: super::support::identity(resource),
        });
    value
}

fn command_mut(value: &mut crate::executor::model::RuntimeBundle) -> &mut BackendCommand {
    let BackendAction::Ffmpeg(command) = &mut value.tasks[0].action else {
        unreachable!()
    };
    command
}

fn file_contract(path: &std::path::Path) -> BackendFilterContract {
    BackendFilterContract::new(
        TOKEN.to_owned(),
        vec![BackendFilterBinding::file(
            TOKEN.to_owned(),
            PathBuf::from(path),
            BackendFilterEscape::Quoted,
        )],
    )
    .unwrap()
}
