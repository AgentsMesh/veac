use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use veac_artifact::ContentDigest;
use veac_codegen::emitter::BackendCapabilityKind;

use super::*;
use crate::RuntimeErrorKind;

struct FakeEnvironment {
    pause: Duration,
}

impl FfmpegEnvironment for FakeEnvironment {
    fn fingerprint(&self) -> Result<FfmpegFingerprint, RuntimeError> {
        std::thread::sleep(self.pause);
        Ok(FfmpegFingerprint {
            version: "fake".to_owned(),
            configuration: ContentDigest::sha256(b"fake"),
        })
    }

    fn encoders(&self) -> Result<BTreeSet<String>, RuntimeError> {
        std::thread::sleep(self.pause);
        Ok(BTreeSet::from(["encoder".to_owned()]))
    }

    fn muxers(&self) -> Result<BTreeSet<String>, RuntimeError> {
        Ok(BTreeSet::from(["muxer".to_owned()]))
    }

    fn execute(&self, _: FfmpegInvocation<'_>) -> Result<(), RuntimeError> {
        Ok(())
    }
}

#[test]
fn invocation_exposes_only_validated_arguments_and_deadline() {
    let arguments = vec!["-version".to_owned()];
    let deadline = future();
    let invocation = FfmpegInvocation::render(&arguments, Path::new("output"), deadline);
    assert_eq!(invocation.arguments(), arguments);
    assert_eq!(invocation.output_root, Some(Path::new("output")));
    assert_eq!(invocation.deadline(), deadline);
}

#[test]
fn default_environment_methods_cover_every_capability_kind() {
    let environment = FakeEnvironment {
        pause: Duration::ZERO,
    };
    assert_eq!(
        environment.fingerprint_until(future()).unwrap().version,
        "fake"
    );
    for (kind, expected) in [
        (BackendCapabilityKind::Encoder, Some("encoder")),
        (BackendCapabilityKind::Muxer, Some("muxer")),
        (BackendCapabilityKind::Decoder, None),
        (BackendCapabilityKind::Demuxer, None),
        (BackendCapabilityKind::Filter, None),
        (BackendCapabilityKind::HardwareBackend, None),
        (BackendCapabilityKind::HardwareDevice, None),
    ] {
        let values = environment.capability_until(kind, future()).unwrap();
        assert_eq!(values.iter().next().map(String::as_str), expected);
    }
}

#[test]
fn default_deadline_checks_run_before_and_after_environment_callbacks() {
    let fast = FakeEnvironment {
        pause: Duration::ZERO,
    };
    let error = fast.fingerprint_until(Instant::now()).unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    let error = fast
        .capability_until(BackendCapabilityKind::Encoder, Instant::now())
        .unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);

    let slow = FakeEnvironment {
        pause: Duration::from_millis(20),
    };
    let error = slow
        .fingerprint_until(Instant::now() + Duration::from_millis(2))
        .unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    let error = slow
        .capability_until(
            BackendCapabilityKind::Encoder,
            Instant::now() + Duration::from_millis(2),
        )
        .unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
}

fn future() -> Instant {
    Instant::now() + Duration::from_secs(2)
}
