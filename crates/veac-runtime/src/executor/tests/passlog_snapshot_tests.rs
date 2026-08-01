use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use veac_artifact::{ArtifactStore, ContentDigest};

use super::support::*;
use crate::executor::{BundleExecutor, FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation};
use crate::RuntimeError;

#[test]
fn fresh_and_resumed_second_passes_consume_verified_private_passlogs() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");
    let original = appended(&output, ".veac-pass");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(SwappingPasslogs {
        calls: Cell::new(0),
        original,
        consumed_prefixes: RefCell::new(Vec::new()),
    });
    let value = bundle(two_pass_tasks("master", &output));

    let first = executor.execute_runtime(&value, &store).unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), b"passlog|tree");
    assert!(first.tasks.iter().all(|task| !task.cache_hit));

    std::fs::write(&output, b"force-second-pass-cache-miss").unwrap();
    let resumed = executor.execute_runtime(&value, &store).unwrap();
    assert!(resumed.tasks[0].cache_hit);
    assert!(!resumed.tasks[1].cache_hit);
    assert_eq!(std::fs::read(&output).unwrap(), b"passlog|tree");
    assert_eq!(executor.environment().calls.get(), 3);

    let prefixes = executor.environment().consumed_prefixes.borrow();
    assert_eq!(prefixes.len(), 2);
    assert!(prefixes
        .iter()
        .all(|prefix| { prefix != &executor.environment().original && !prefix.exists() }));
}

struct SwappingPasslogs {
    calls: Cell<usize>,
    original: PathBuf,
    consumed_prefixes: RefCell<Vec<PathBuf>>,
}

impl FfmpegEnvironment for SwappingPasslogs {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        Ok(FfmpegFingerprint {
            version: "ffmpeg version passlog-snapshot-test".to_owned(),
            configuration: ContentDigest::sha256(b"passlog-snapshot-test"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::new())
    }

    fn execute(&self, invocation: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        self.calls.set(self.calls.get() + 1);
        let arguments = invocation.arguments();
        let prefix = PathBuf::from(option(arguments, "-passlogfile").unwrap());
        if option(arguments, "-pass") == Some("1") {
            std::fs::write(appended(&prefix, "-0.log"), b"passlog").unwrap();
            std::fs::write(appended(&prefix, "-0.log.mbtree"), b"tree").unwrap();
            return Ok(());
        }
        self.consumed_prefixes.borrow_mut().push(prefix.clone());
        if self.calls.get() == 2 {
            assert!(!appended(&self.original, "-0.log").exists());
            assert!(!appended(&self.original, "-0.log.mbtree").exists());
        }
        self.swap_originals(b"attacker-log", b"attacker-tree");
        let log = std::fs::read(appended(&prefix, "-0.log")).unwrap();
        let tree = std::fs::read(appended(&prefix, "-0.log.mbtree")).unwrap();
        self.swap_originals(b"passlog", b"tree");
        let mut consumed = log;
        consumed.push(b'|');
        consumed.extend(tree);
        std::fs::write(arguments.last().unwrap(), consumed).unwrap();
        Ok(())
    }
}

impl SwappingPasslogs {
    fn swap_originals(&self, log: &[u8], tree: &[u8]) {
        std::fs::write(appended(&self.original, "-0.log"), log).unwrap();
        std::fs::write(appended(&self.original, "-0.log.mbtree"), tree).unwrap();
    }
}

fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn appended(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}
