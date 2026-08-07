use std::time::{Duration, Instant};
use tempfile::tempdir;
use veac_ir::{StreamChoice, StreamIntent};

use crate::environment::{Environment, SystemEnvironment};

#[test]
fn system_environment_uses_runtime_identity_probe_and_queries() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("bytes.bin");
    std::fs::write(&file, "bytes").unwrap();
    let environment = SystemEnvironment::default();
    let identity = environment.identity(&file).unwrap();
    assert_eq!(identity.digest.len(), 64);
    let encoders = environment.ffmpeg_encoders().unwrap();
    assert!(encoders.contains("libx264") || encoders.contains("h264_videotoolbox"));
    assert!(environment.ffmpeg_decoders().unwrap().contains("h264"));
    assert!(environment.ffmpeg_demuxers().unwrap().contains("mov"));
    assert!(environment.ffmpeg_filters().unwrap().contains("scale"));
    environment.ffmpeg_hardware_backends().unwrap();
    environment.ffmpeg_hardware_devices().unwrap();
    let fingerprint = environment.ffmpeg_fingerprint().unwrap();
    assert!(fingerprint.version.starts_with("ffmpeg version"));
    fingerprint.configuration.validate().unwrap();

    let missing = temp.path().join("missing.mp4");
    let error = environment
        .probe(
            &missing,
            StreamIntent {
                video: StreamChoice::Auto,
                audio: StreamChoice::Auto,
            },
        )
        .unwrap_err();
    assert!(error.to_string().contains("PROBE_FAILED"));
    assert!(environment
        .identity(&missing)
        .unwrap_err()
        .to_string()
        .contains("IDENTITY_FAILED"));
}

#[cfg(unix)]
#[test]
fn system_environment_reuses_one_pinned_ffmpeg_across_queries() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let binary = temp.path().join("ffmpeg");
    std::fs::write(
        &binary,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  printf '#!/bin/sh\\necho replaced\\n' > '{}'\n  printf 'ffmpeg version cli-pinned\\n'\n  rm \"$0\"\n  exit 0\nfi\nif [ \"$2\" = \"-muxers\" ]; then\n  printf ' E mxf fixture\\n'\nelse\n  printf ' V..... cli_encoder fixture\\n'\nfi\nrm \"$0\"\n",
            binary.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let environment = SystemEnvironment::new(&binary);

    assert_eq!(
        environment.ffmpeg_fingerprint().unwrap().version,
        "ffmpeg version cli-pinned"
    );
    assert!(environment
        .ffmpeg_encoders()
        .unwrap()
        .contains("cli_encoder"));
    assert!(environment.ffmpeg_muxers().unwrap().contains("mxf"));
}

#[cfg(unix)]
#[test]
fn system_environment_reuses_one_pinned_ffprobe_across_media() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let media = temp.path().join("media.bin");
    let binary = temp.path().join("ffprobe");
    std::fs::write(&media, b"stable").unwrap();
    std::fs::write(
        &binary,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  printf '#!/bin/sh\\nexit 97\\n' > '{}'\n  chmod 700 '{}'\n  printf 'ffprobe version cli-pinned\\n'\n  rm \"$0\"\n  exit 0\nfi\nprintf '%s' '{{\"streams\":[],\"format\":{{\"format_name\":\"data\"}}}}'\nrm \"$0\"\n",
            binary.display(),
            binary.display(),
        ),
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let environment = SystemEnvironment::with_tools("unused", &binary);
    let intent = StreamIntent {
        video: StreamChoice::Auto,
        audio: StreamChoice::Auto,
    };

    for _ in 0..2 {
        let snapshot = environment
            .probe_until(
                &media,
                intent.clone(),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        assert!(snapshot
            .engine
            .starts_with("ffprobe version cli-pinned sha256:"));
    }
    assert!(std::fs::read_to_string(&binary)
        .unwrap()
        .contains("exit 97"));
}

#[test]
fn canonical_file_requires_an_existing_regular_file() {
    let temp = tempdir().unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, "x").unwrap();
    assert!(crate::fs::canonical_file(&file, "fixture")
        .unwrap()
        .is_absolute());
    assert!(crate::fs::canonical_file(temp.path(), "fixture")
        .unwrap_err()
        .to_string()
        .contains("NOT_A_FILE"));
    assert!(
        crate::fs::canonical_file(&temp.path().join("missing"), "fixture")
            .unwrap_err()
            .to_string()
            .contains("PATH_UNAVAILABLE")
    );
}

#[test]
fn system_environment_maps_segment_validation_failures() {
    use veac_artifact::{ArtifactRecord, ContentDigest};

    let temp = tempdir().unwrap();
    let project = super::support::canonical_project(&temp, super::support::GENERATED_SOURCE);
    let fixture_environment = super::support::FakeEnvironment::success();
    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &fixture_environment)
            .unwrap();
    let contract =
        super::substitution_command_tests::segment_contract(&prepared, &fixture_environment);
    let record = ArtifactRecord {
        key: ContentDigest::sha256(b"missing-segment"),
        content: ContentDigest::sha256(b"missing"),
        size_bytes: 7,
    };
    let error = SystemEnvironment::default()
        .validate_render_segment(
            &temp.path().join("missing.mp4"),
            &contract,
            &record,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("RENDER_SEGMENT_POSTFLIGHT_FAILED"));
}

#[test]
fn system_environment_executes_a_guarded_render_bundle() {
    let temp = tempdir().unwrap();
    let project = super::support::canonical_project(&temp, super::support::GENERATED_SOURCE);
    crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        crate::arguments::SubstitutionPolicy::Original,
        crate::arguments::SubstitutionPolicy::Original,
        &SystemEnvironment::default(),
    )
    .unwrap();
    assert!(temp.path().join("render.mp4").is_file());
}
