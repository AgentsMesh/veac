use std::collections::{BTreeMap, BTreeSet};

use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_runtime::executor::{
    BundleExecutor, FfmpegEnvironment, FfmpegFingerprint, FfmpegInvocation, SystemFfmpeg,
};
use veac_runtime::RuntimeError;

use super::hls::{assert_no_staging_residue, hls_project};
use super::support::*;

#[test]
fn failed_hls_rerender_preserves_the_complete_previous_package() {
    let temp = tempdir().unwrap();
    let tone = tone_fixture(temp.path(), "hls-failure-tone", 440);
    let assets = BTreeMap::from([("med_hls_tone".to_owned(), tone)]);
    let first = prepare_delivery(hls_project(), &assets, temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    first.execute(&store);
    let root = first.path("dlv_hls");
    let before = package_bytes(root);

    let mut changed = hls_project();
    let ClipSource::Generated {
        generator: Generator::Solid { color },
    } = &mut changed.project.sequences[0].tracks[0].clips[0].source
    else {
        panic!("expected solid HLS fixture")
    };
    *color = super::support::color(10, 200, 30);
    let changed = prepare_delivery(changed, &assets, temp.path());
    let error = BundleExecutor::new(FailingFfmpeg::default())
        .execute(&changed.bundle, &store)
        .unwrap_err();

    assert!(error.message.contains("injected HLS render failure"));
    assert_eq!(package_bytes(root), before);
    assert_no_staging_residue(temp.path());
}

fn package_bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            let bytes = std::fs::read(entry.path()).unwrap();
            (name, bytes)
        })
        .collect()
}

#[derive(Default)]
struct FailingFfmpeg {
    inner: SystemFfmpeg,
}

impl FfmpegEnvironment for FailingFfmpeg {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        self.inner.fingerprint()
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.encoders()
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.muxers()
    }

    fn decoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.decoders()
    }

    fn demuxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.demuxers()
    }

    fn filters(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.filters()
    }

    fn hardware_backends(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.hardware_backends()
    }

    fn hardware_devices(&self) -> Result<BTreeSet<String>, RuntimeError> {
        self.inner.hardware_devices()
    }

    fn execute(&self, _: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        Err(RuntimeError::new("injected HLS render failure"))
    }
}
