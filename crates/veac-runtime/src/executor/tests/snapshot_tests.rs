use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::PathBuf;

use veac_artifact::{ArtifactStore, ContentDigest};
use veac_codegen::emitter::{BackendAction, BackendInput};

use super::support::*;
use crate::executor::{BundleExecutor, FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

#[test]
fn transient_source_swap_consumes_a_private_snapshot_and_keeps_identity_stable() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "input.bin");
    let output = path(temp.path(), "output.bin");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    std::fs::write(&input, b"version-a").unwrap();
    let mut task = video_task("snapshot", &output);
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command.inputs.push(BackendInput {
        path: input.clone(),
    });
    let mut value = bundle(vec![task]);
    value.protected_resources.push(protected_resource(&input));
    let executor = BundleExecutor::new(SwappingFfmpeg {
        original: input.clone(),
        invocations: RefCell::new(Vec::new()),
    });

    let first = executor.execute_runtime(&value, &store).unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), b"version-a");
    assert_eq!(std::fs::read(&input).unwrap(), b"version-a");
    let snapshot = executor.environment().invocations.borrow()[0].clone();
    assert_ne!(snapshot, input);
    assert!(!snapshot.exists(), "snapshot TempDir must be removed");

    let second = executor.execute_runtime(&value, &store).unwrap();
    assert!(second.tasks[0].cache_hit);
    assert_eq!(executor.environment().invocations.borrow().len(), 1);
    assert_eq!(
        first.tasks[0].checkpoint.key,
        second.tasks[0].checkpoint.key
    );
}

struct SwappingFfmpeg {
    original: PathBuf,
    invocations: RefCell<Vec<PathBuf>>,
}

impl FfmpegEnvironment for SwappingFfmpeg {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        Ok(FfmpegFingerprint {
            version: "ffmpeg version snapshot-test".to_owned(),
            configuration: ContentDigest::sha256(b"snapshot-test"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        let arguments = invocation.arguments();
        let input = arguments
            .windows(2)
            .find(|pair| pair[0] == "-i")
            .map(|pair| PathBuf::from(&pair[1]))
            .unwrap();
        self.invocations.borrow_mut().push(input.clone());
        std::fs::write(&self.original, b"version-b").unwrap();
        let consumed = std::fs::read(input).unwrap();
        std::fs::write(&self.original, b"version-a").unwrap();
        std::fs::write(arguments.last().unwrap(), consumed).unwrap();
        Ok(())
    }
}
