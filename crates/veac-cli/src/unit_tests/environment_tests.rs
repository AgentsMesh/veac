use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use tempfile::tempdir;
use veac_artifact::{ArtifactRecord, ContentDigest};
use veac_ir::{MediaIdentity, MediaProbeSnapshot, StreamIntent};
use veac_runtime::executor::{FfmpegFingerprint, FfmpegInvocation};

use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};
use crate::environment::Environment;

#[test]
fn default_segment_validation_checks_payload_probe_and_contract() {
    let fixture = SegmentFixture::new();
    let environment = DefaultPostflight(&fixture.environment);
    environment
        .validate_render_segment(
            &fixture.payload,
            &fixture.contract,
            &fixture.record,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
}

#[test]
fn default_segment_validation_enforces_deadline_size_and_record_limits() {
    let fixture = SegmentFixture::new();
    let environment = DefaultPostflight(&fixture.environment);
    let expired = environment
        .validate_render_segment(
            &fixture.payload,
            &fixture.contract,
            &fixture.record,
            Instant::now(),
        )
        .unwrap_err();
    assert!(expired.is_resource_limit());

    let mut oversized = fixture.record.clone();
    oversized.size_bytes = veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES + 1;
    let oversized = environment
        .validate_render_segment(
            &fixture.payload,
            &fixture.contract,
            &oversized,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap_err();
    assert!(oversized.is_resource_limit());

    let mut wrong_size = fixture.record.clone();
    wrong_size.size_bytes += 1;
    let wrong_size = environment
        .validate_render_segment(
            &fixture.payload,
            &fixture.contract,
            &wrong_size,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap_err();
    assert!(!wrong_size.is_resource_limit());
    assert!(wrong_size.to_string().contains("payload size differs"));
}

struct SegmentFixture {
    _temp: tempfile::TempDir,
    payload: std::path::PathBuf,
    contract: veac_artifact::FullRenderSegmentContract,
    record: ArtifactRecord,
    environment: FakeEnvironment,
}

impl SegmentFixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let source = GENERATED_SOURCE.replace("200ms", "1s");
        let project = canonical_project(&temp, &source);
        let environment = FakeEnvironment::success();
        let prepared = crate::planning::prepare(&project, None, &environment).unwrap();
        let contract = super::substitution_command_tests::segment_contract(&prepared, &environment);
        let payload = temp.path().join("segment.mp4");
        std::fs::write(&payload, b"rendered").unwrap();
        let content = ContentDigest::sha256(b"rendered");
        let record = ArtifactRecord {
            key: veac_artifact::artifact_key(contract.descriptor()).unwrap(),
            content,
            size_bytes: 8,
        };
        Self {
            _temp: temp,
            payload,
            contract,
            record,
            environment,
        }
    }
}

struct DefaultPostflight<'a>(&'a FakeEnvironment);

impl Environment for DefaultPostflight<'_> {
    fn identity(&self, path: &Path) -> crate::CliResult<MediaIdentity> {
        self.0.identity(path)
    }

    fn probe(&self, path: &Path, intent: StreamIntent) -> crate::CliResult<MediaProbeSnapshot> {
        self.0.probe(path, intent)
    }

    fn ffmpeg_encoders(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_encoders()
    }

    fn ffmpeg_muxers(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_muxers()
    }

    fn ffmpeg_decoders(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_decoders()
    }

    fn ffmpeg_demuxers(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_demuxers()
    }

    fn ffmpeg_filters(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_filters()
    }

    fn ffmpeg_hardware_backends(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_hardware_backends()
    }

    fn ffmpeg_hardware_devices(&self) -> crate::CliResult<BTreeSet<String>> {
        self.0.ffmpeg_hardware_devices()
    }

    fn execute_ffmpeg(&self, invocation: FfmpegInvocation<'_>) -> crate::CliResult {
        self.0.execute_ffmpeg(invocation)
    }

    fn ffmpeg_fingerprint(&self) -> crate::CliResult<FfmpegFingerprint> {
        self.0.ffmpeg_fingerprint()
    }
}
