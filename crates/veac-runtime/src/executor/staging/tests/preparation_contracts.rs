use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use veac_codegen::emitter::{
    BackendAction, BackendCommand, BackendFilterBinding, BackendFilterContract,
    BackendFilterEscape, BackendInternalAccess, BackendOutput, BackendPhase, BackendPreparation,
    BackendProduct, BackendTask,
};

use super::super::perform;
use crate::executor::tests::support::FakeFfmpeg;
use crate::executor::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";

#[test]
fn preparations_produce_sidecars_before_the_main_command_consumes_them() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("master.mp4");
    let sidecar = PathBuf::from("motion/transforms.trf");
    let preparation = command(internal(&sidecar, BackendInternalAccess::Produce, "result"));
    let main = command(internal(&sidecar, BackendInternalAccess::Consume, "input"));
    let task = BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_preparation_test").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::VideoMaster,
        output: BackendOutput::File(output),
        action: BackendAction::Ffmpeg(BackendCommand {
            preparations: vec![BackendPreparation {
                command: preparation,
                outputs: vec![sidecar],
            }],
            ..main
        }),
    };
    let environment = SidecarFfmpeg::default();
    let staged = perform(&environment, &task, deadline()).unwrap();
    assert_eq!(environment.inner.calls.borrow().len(), 2);
    assert!(environment.inner.calls.borrow()[1]
        .iter()
        .any(|value| value.contains("input=") && value.contains("transforms.trf")));
    drop(staged);
}

#[derive(Default)]
struct SidecarFfmpeg {
    inner: FakeFfmpeg,
}

impl FfmpegEnvironment for SidecarFfmpeg {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        self.inner.fingerprint()
    }
    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.encoders()
    }
    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.muxers()
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        for value in invocation.arguments() {
            if let Some(path) = value
                .split("result='")
                .nth(1)
                .and_then(|tail| tail.split('\'').next())
            {
                std::fs::write(path, b"sidecar")
                    .map_err(|error| RuntimeError::new(error.to_string()))?;
            }
        }
        self.inner.execute(invocation)
    }
}

fn command(contract: BackendFilterContract) -> BackendCommand {
    BackendCommand {
        preparations: Vec::new(),
        inputs: Vec::new(),
        filter_graph: Some(contract.render_original().unwrap()),
        filter_contract: Some(contract),
        maps: Vec::new(),
        output_args: vec!["-f".to_owned(), "data".to_owned()],
        output_path: PathBuf::from("ignored.null"),
    }
}

fn internal(path: &Path, access: BackendInternalAccess, option: &str) -> BackendFilterContract {
    BackendFilterContract::new(
        format!("{option}='{TOKEN}'"),
        vec![BackendFilterBinding::internal_file(
            TOKEN.to_owned(),
            path.to_path_buf(),
            access,
            BackendFilterEscape::Quoted,
        )],
    )
    .unwrap()
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
