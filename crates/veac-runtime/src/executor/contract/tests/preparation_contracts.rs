use std::path::{Path, PathBuf};

use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendFilterBinding, BackendFilterContract,
    BackendFilterEscape, BackendInput, BackendInternalAccess, BackendPreparation,
};

use super::super::filter;
use super::support::{bundle, identity};

const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";
const SECOND_TOKEN: &str = "__VEAC_FILTER_RESOURCE_0001__";

#[test]
fn preparation_sidecars_follow_producer_consumer_order() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.mp4");
    std::fs::write(&source, b"source").unwrap();
    let output = temp.path().join("output.mp4");
    let mut value = bundle(&output);
    value
        .protected_resources
        .push(veac_codegen::emitter::BackendResource {
            path: source.clone(),
            expected_identity: identity(&source),
        });
    let sidecar = PathBuf::from("motion/transforms.trf");
    let preparation = command(&source, internal(&sidecar, BackendInternalAccess::Produce));
    let main = command(&source, internal(&sidecar, BackendInternalAccess::Consume));
    value.tasks[0].action = BackendAction::Ffmpeg(BackendCommand {
        preparations: vec![BackendPreparation {
            command: preparation,
            outputs: vec![sidecar],
        }],
        ..main
    });
    filter::validate(&value).unwrap();
}

#[test]
fn preparations_reject_duplicate_internal_producers() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.mp4");
    std::fs::write(&source, b"source").unwrap();
    let output = temp.path().join("output.mp4");
    let mut value = bundle(&output);
    value
        .protected_resources
        .push(veac_codegen::emitter::BackendResource {
            path: source.clone(),
            expected_identity: identity(&source),
        });
    let sidecar = PathBuf::from("motion/transforms.trf");
    let producer = BackendFilterContract::new(
        format!("{TOKEN};{SECOND_TOKEN}"),
        vec![
            BackendFilterBinding::internal_file(
                TOKEN.to_owned(),
                sidecar.clone(),
                BackendInternalAccess::Produce,
                BackendFilterEscape::Quoted,
            ),
            BackendFilterBinding::internal_file(
                SECOND_TOKEN.to_owned(),
                sidecar.clone(),
                BackendInternalAccess::Produce,
                BackendFilterEscape::Quoted,
            ),
        ],
    )
    .unwrap();
    let main = command(&source, internal(&sidecar, BackendInternalAccess::Consume));
    value.tasks[0].action = BackendAction::Ffmpeg(BackendCommand {
        preparations: vec![BackendPreparation {
            command: command(&source, producer),
            outputs: vec![sidecar],
        }],
        ..main
    });
    let error = filter::validate(&value).unwrap_err();
    assert!(error.message.contains("exactly one internal producer"));
}

#[test]
fn preparations_reject_nesting() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.mp4");
    std::fs::write(&source, b"source").unwrap();
    let output = temp.path().join("output.mp4");
    let mut value = bundle(&output);
    value
        .protected_resources
        .push(veac_codegen::emitter::BackendResource {
            path: source.clone(),
            expected_identity: identity(&source),
        });
    let sidecar = PathBuf::from("transforms.trf");
    let mut preparation = command(&source, internal(&sidecar, BackendInternalAccess::Consume));
    preparation.preparations.push(BackendPreparation {
        command: command(&source, internal(&sidecar, BackendInternalAccess::Produce)),
        outputs: vec![sidecar.clone()],
    });
    let main = command(&source, internal(&sidecar, BackendInternalAccess::Consume));
    value.tasks[0].action = BackendAction::Ffmpeg(BackendCommand {
        preparations: vec![BackendPreparation {
            command: preparation,
            outputs: vec![PathBuf::from("../escape.trf")],
        }],
        ..main
    });
    let error = filter::validate(&value).unwrap_err();
    assert!(error.message.contains("nested preparations"));
}

fn command(source: &Path, contract: BackendFilterContract) -> BackendCommand {
    BackendCommand {
        preparations: Vec::new(),
        inputs: vec![BackendInput {
            path: source.to_path_buf(),
        }],
        filter_graph: Some(contract.render_original().unwrap()),
        filter_contract: Some(contract),
        maps: Vec::new(),
        output_args: Vec::new(),
        output_path: PathBuf::from("ignored.null"),
    }
}

fn internal(path: &Path, access: BackendInternalAccess) -> BackendFilterContract {
    BackendFilterContract::new(
        TOKEN.to_owned(),
        vec![BackendFilterBinding::internal_file(
            TOKEN.to_owned(),
            path.to_path_buf(),
            access,
            BackendFilterEscape::Quoted,
        )],
    )
    .unwrap()
}
