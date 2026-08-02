use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

use super::super::{ffmpeg, perform, StaleFamily};
use super::deadline;
use crate::executor::tests::support::{command, FakeFfmpeg};
use crate::executor::{FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

#[test]
fn ffmpeg_invocation_receives_the_exact_task_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("master.mp4");
    let task = task(
        BackendOutput::File(output.clone()),
        BackendPhase::Single,
        BackendProduct::VideoMaster,
        command(&output),
    );
    let environment = FakeFfmpeg::default();
    let expected = deadline();
    let staged = perform(&environment, &task, expected).unwrap();
    assert_eq!(environment.deadlines.borrow().as_slice(), &[expected]);
    drop(staged);
    assert!(!output.exists());
}

#[test]
fn image_sequence_staging_maps_frames_and_defers_stale_enumeration() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let descriptor = super::super::directory::Directory::open(staging.path()).unwrap();
    let pattern = temp.path().join("frame-%d.png");
    std::fs::write(temp.path().join("frame-99.png"), b"stale").unwrap();
    let mut value = command(&pattern);
    let (files, stale) = ffmpeg::stage_sequence(
        &FakeFfmpeg::default(),
        &staging,
        &descriptor,
        &mut value,
        &pattern,
        deadline(),
    )
    .unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(stale, vec![StaleFamily::ImageSequence(pattern)]);
    assert!(files.iter().all(|file| file.source.is_file()));
}

#[test]
fn multi_file_staging_exposes_a_missing_declared_output() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("left.wav");
    let second = temp.path().join("right.wav");
    let task = task(
        BackendOutput::Files {
            paths: vec![first, second],
        },
        BackendPhase::Single,
        BackendProduct::AudioStem,
        command(temp.path().join("left.wav").as_path()),
    );
    let error = perform(&FakeFfmpeg::default(), &task, deadline())
        .err()
        .unwrap();
    assert!(error.message.contains("render output") && error.message.contains("unavailable"));
}

#[test]
fn passlog_staging_requires_the_primary_declared_log() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let descriptor = super::super::directory::Directory::open(staging.path()).unwrap();
    let prefix = temp.path().join("master.pass");
    let expected = appended(&prefix, "-0.log");
    let mut value = command(&temp.path().join("first.null"));
    value.output_args.extend([
        "-pass".to_owned(),
        "1".to_owned(),
        "-passlogfile".to_owned(),
        prefix.to_string_lossy().into_owned(),
    ]);
    let task = task(
        BackendOutput::File(expected.clone()),
        BackendPhase::FirstPass,
        BackendProduct::RenderPassLog,
        value.clone(),
    );
    let environment = MissingPrimary {
        inner: FakeFfmpeg::default(),
        staged_log: staging.path().join("passlog-0.log"),
    };
    let error = ffmpeg::stage_passlog(
        &environment,
        &staging,
        &descriptor,
        &task,
        &mut value,
        &expected,
        deadline(),
    )
    .err()
    .unwrap();
    assert!(error
        .message
        .contains("did not produce its declared passlog"));
}

struct MissingPrimary {
    inner: FakeFfmpeg,
    staged_log: PathBuf,
}

impl FfmpegEnvironment for MissingPrimary {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        self.inner.fingerprint()
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.encoders()
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        self.inner.execute(invocation)?;
        std::fs::remove_file(&self.staged_log).map_err(|error| RuntimeError::new(error.to_string()))
    }
}

fn task(
    output: BackendOutput,
    phase: BackendPhase,
    product: BackendProduct,
    command: veac_codegen::emitter::BackendCommand,
) -> BackendTask {
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_staging_test").unwrap(),
        phase,
        product,
        output,
        action: BackendAction::Ffmpeg(command),
    }
}

fn appended(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
