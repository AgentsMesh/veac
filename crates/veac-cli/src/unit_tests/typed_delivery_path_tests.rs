use tempfile::{tempdir, TempDir};
use veac_ir::*;

use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE};

#[test]
fn typed_file_targets_bind_mp3_gif_and_still_paths() {
    let cases = [
        ("mix.mp3", mp3()),
        ("preview.gif", gif()),
        ("poster.png", still()),
    ];
    for (name, kind) in cases {
        let temp = tempdir().unwrap();
        let mut prepared = generated(&temp);
        set_target(
            &mut prepared,
            DeliverableTarget::File { name: name.into() },
            kind,
        );
        let outputs = crate::output::bind_render_outputs(&mut prepared, None).unwrap();
        assert_eq!(
            outputs,
            vec![temp.path().canonicalize().unwrap().join(name)]
        );
    }
}

#[test]
fn hls_package_accepts_a_directory_and_rejects_file_or_symlink_roots() {
    let temp = tempdir().unwrap();
    let stream = temp.path().join("stream");
    std::fs::create_dir(&stream).unwrap();
    let mut prepared = generated_hls(&temp, "stream");
    assert_eq!(
        crate::output::bind_render_outputs(&mut prepared, None).unwrap(),
        vec![stream.canonicalize().unwrap()]
    );

    std::fs::remove_dir(&stream).unwrap();
    std::fs::write(&stream, b"not a package").unwrap();
    let error =
        crate::output::bind_render_outputs(&mut generated_hls(&temp, "stream"), None).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_IS_FILE"));

    #[cfg(unix)]
    {
        std::fs::remove_file(&stream).unwrap();
        let real = temp.path().join("real-package");
        std::fs::create_dir(&real).unwrap();
        std::os::unix::fs::symlink(&real, &stream).unwrap();
        let error = crate::output::bind_render_outputs(&mut generated_hls(&temp, "stream"), None)
            .unwrap_err();
        assert!(error.to_string().contains("OUTPUT_SYMLINK"));
    }
}

#[test]
fn hls_package_cannot_consume_an_input_or_the_artifact_store() {
    let temp = tempdir().unwrap();
    let stream = temp.path().join("stream");
    std::fs::create_dir(&stream).unwrap();
    std::fs::write(stream.join("clip.mp4"), b"input").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].source = MaterialSource::File {
        uri: "stream/clip.mp4".into(),
    };
    std::fs::write(&project, veac_ir::canonical_json(&envelope).unwrap()).unwrap();
    let mut prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    set_target(
        &mut prepared,
        DeliverableTarget::Package {
            name: "stream".into(),
        },
        hls(),
    );
    let error = crate::output::bind_render_outputs(&mut prepared, None).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));

    let store = temp.path().join(".veac-artifacts");
    std::fs::create_dir(&store).unwrap();
    let error =
        crate::output::bind_render_outputs(&mut generated_hls(&temp, ".veac-artifacts"), None)
            .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_RESERVED_DIRECTORY"));
}

#[test]
fn package_ancestor_of_the_project_and_store_is_rejected() {
    let temp = tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let name = root.file_name().unwrap().to_str().unwrap();
    let destination = root.parent().unwrap();
    let mut prepared = generated_hls(&temp, name);
    let error = crate::output::bind_render_outputs(&mut prepared, Some(destination)).unwrap_err();
    assert!(
        error.to_string().contains("OUTPUT_OVERWRITES_INPUT"),
        "unexpected output guard: {error}"
    );
}

fn generated(temp: &TempDir) -> crate::planning::PreparedPlan {
    let project = canonical_project(temp, GENERATED_SOURCE);
    crate::planning::prepare_with_material_root(&project, None, None, &FakeEnvironment::success())
        .unwrap()
}

fn generated_hls(temp: &TempDir, name: &str) -> crate::planning::PreparedPlan {
    let mut prepared = generated(temp);
    set_target(
        &mut prepared,
        DeliverableTarget::Package { name: name.into() },
        hls(),
    );
    prepared
}

fn set_target(
    prepared: &mut crate::planning::PreparedPlan,
    target: DeliverableTarget,
    kind: DeliverableKind,
) {
    let deliverable = &mut prepared.plan.output.deliverables[0];
    deliverable.target = target;
    deliverable.kind = kind;
}

fn mp3() -> DeliverableKind {
    DeliverableKind::AudioFile(AudioFile {
        source: AudioMixSource::Master,
        encoding: AudioFileEncoding::Mp3(Mp3Encoding {
            bitrate_bps: 128_000,
            sample_rate_hz: 48_000,
            channel_layout: AudioChannelLayout::Stereo,
        }),
    })
}

fn gif() -> DeliverableKind {
    DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
        playback: GifPlayback::Forever,
        dither: GifDither::Sierra2,
    }))
}

fn still() -> DeliverableKind {
    DeliverableKind::StillImage(StillImage {
        frame: FrameSelection::Containing {
            at: RationalTime::zero(600).unwrap(),
        },
        encoding: ImageFormat::Png,
    })
}

fn hls() -> DeliverableKind {
    DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(HlsPackage {
        segment_duration: RationalTime::new(1200, 600).unwrap(),
        audio: None,
        renditions: vec![HlsRendition {
            id: HlsRenditionId::new("rnd_main").unwrap(),
            raster: HlsRenditionRaster {
                width: 640,
                height: 360,
            },
            encoding: HlsVideoEncoding::H264(HlsH264Encoding {
                rate_control: HlsCappedBitrate {
                    target_bps: 800_000,
                    max_bps: 900_000,
                    buffer_size_bits: 1_600_000,
                },
                profile: Some(HlsH264Profile::Main),
                level: Some("3.1".into()),
                color_space: None,
                b_frames: Some(2),
            }),
        }],
    }))
}
