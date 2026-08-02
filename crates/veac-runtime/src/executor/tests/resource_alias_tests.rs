use std::collections::BTreeSet;
use std::path::PathBuf;

use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{BackendAction, BackendInput};

use super::support::*;
use crate::executor::{BundleExecutor, FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

#[test]
fn parent_symlink_retarget_to_same_bytes_fails_resource_fingerprint() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let first = path(temp.path(), "first");
    let second = path(temp.path(), "second");
    std::fs::create_dir(&first).unwrap();
    std::fs::create_dir(&second).unwrap();
    std::fs::write(first.join("input.bin"), b"same-bytes").unwrap();
    std::fs::write(second.join("input.bin"), b"same-bytes").unwrap();
    let link = path(temp.path(), "current");
    symlink(&first, &link).unwrap();
    let input = link.join("input.bin");
    let output = path(temp.path(), "output.bin");
    let mut task = video_task("master", &output);
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        panic!("test task must use FFmpeg")
    };
    command.inputs.push(BackendInput {
        path: input.clone(),
    });
    let mut value = bundle(vec![task]);
    value.protected_resources.push(protected_resource(&input));
    let executor = BundleExecutor::new(RetargetingFfmpeg {
        inner: FakeFfmpeg::default(),
        link,
        target: second,
    });
    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("resource paths changed"));
    assert_eq!(executor.environment().inner.calls.borrow().len(), 1);
    assert!(!output.exists());
}

struct RetargetingFfmpeg {
    inner: FakeFfmpeg,
    link: PathBuf,
    target: PathBuf,
}

impl FfmpegEnvironment for RetargetingFfmpeg {
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
        use std::os::unix::fs::symlink;

        self.inner.execute(invocation)?;
        std::fs::remove_file(&self.link).unwrap();
        symlink(&self.target, &self.link).unwrap();
        Ok(())
    }
}
