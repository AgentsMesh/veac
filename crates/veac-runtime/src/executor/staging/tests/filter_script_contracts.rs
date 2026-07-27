use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::PathBuf;

use veac_artifact::ContentDigest;
use veac_codegen::emitter::{BackendAction, MAX_INLINE_FILTER_GRAPH_BYTES};

use super::super::perform;
use super::deadline;
use crate::executor::tests::support::{video_task, FakeFfmpeg};
use crate::executor::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

#[test]
fn small_graphs_stay_inline_without_a_sidecar() {
    let temp = tempfile::tempdir().unwrap();
    let mut task = video_task("inline", &temp.path().join("inline.bin"));
    command(&mut task).filter_graph = Some("null[outv]".to_owned());
    let environment = Observing::new(Outcome::Success);
    let staged = perform(&environment, &task, deadline()).unwrap();
    let arguments = &environment.inner.calls.borrow()[0];
    assert!(option(arguments, "-filter_complex").is_some());
    assert!(option(arguments, "-filter_complex_script").is_none());
    assert!(environment.script_path.borrow().is_none());
    drop(staged);
}

#[test]
fn large_graphs_use_an_exact_ephemeral_staging_script() {
    let temp = tempfile::tempdir().unwrap();
    let mut task = video_task("script", &temp.path().join("script.bin"));
    let graph = "x".repeat(MAX_INLINE_FILTER_GRAPH_BYTES + 1);
    command(&mut task).filter_graph = Some(graph.clone());
    let environment = Observing::new(Outcome::Success);
    let staged = perform(&environment, &task, deadline()).unwrap();
    let path = environment.script_path.borrow().clone().unwrap();
    assert_eq!(
        environment.script_bytes.borrow().as_deref(),
        Some(graph.as_bytes())
    );
    assert!(!path.exists(), "sidecar must be removed before commit");
    let arguments = &environment.inner.calls.borrow()[0];
    assert_eq!(option(arguments, "-filter_complex_script"), path.to_str());
    assert!(option(arguments, "-filter_complex").is_none());
    assert!(staged.files().iter().all(|file| file.source != path));
}

#[test]
fn scripts_are_cleaned_after_execution_failure_or_mutation() {
    for outcome in [Outcome::Failure, Outcome::Mutate] {
        let temp = tempfile::tempdir().unwrap();
        let mut task = video_task("failure", &temp.path().join("failure.bin"));
        command(&mut task).filter_graph = Some("x".repeat(MAX_INLINE_FILTER_GRAPH_BYTES + 1));
        let environment = Observing::new(outcome);
        let error = perform(&environment, &task, deadline()).err().unwrap();
        let path = environment.script_path.borrow().clone().unwrap();
        assert!(
            !path.exists(),
            "failed task leaked sidecar at {}",
            path.display()
        );
        match outcome {
            Outcome::Failure => assert!(error.message.contains("injected FFmpeg failure")),
            Outcome::Mutate => assert!(error.message.contains("filter script")),
            Outcome::Success => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
enum Outcome {
    Success,
    Failure,
    Mutate,
}

struct Observing {
    inner: FakeFfmpeg,
    outcome: Outcome,
    script_path: RefCell<Option<PathBuf>>,
    script_bytes: RefCell<Option<Vec<u8>>>,
}

impl Observing {
    fn new(outcome: Outcome) -> Self {
        Self {
            inner: FakeFfmpeg::default(),
            outcome,
            script_path: RefCell::new(None),
            script_bytes: RefCell::new(None),
        }
    }
}

impl FfmpegEnvironment for Observing {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        Ok(FfmpegFingerprint {
            version: "observed".to_owned(),
            configuration: ContentDigest::sha256(b"observed"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        if let Some(value) = option(invocation.arguments(), "-filter_complex_script") {
            let path = PathBuf::from(value);
            *self.script_bytes.borrow_mut() = Some(std::fs::read(&path).unwrap());
            *self.script_path.borrow_mut() = Some(path.clone());
            if matches!(self.outcome, Outcome::Mutate) {
                std::fs::write(path, b"changed").unwrap();
            }
        }
        if matches!(self.outcome, Outcome::Failure) {
            Err(RuntimeError::new("injected FFmpeg failure"))
        } else {
            self.inner.execute(invocation)
        }
    }
}

fn command(
    task: &mut veac_codegen::emitter::BackendTask,
) -> &mut veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command
}

fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}
