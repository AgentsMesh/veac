use super::*;
use veac_ir::*;

#[test]
fn extended_delivery_names_and_target_kinds_are_typed() {
    let cases = [
        (
            "mix.MP3",
            DeliverableKind::AudioFile(AudioFile {
                source: AudioMixSource::Master,
                encoding: AudioFileEncoding::Mp3(Mp3Encoding {
                    bitrate_bps: 128_000,
                    sample_rate_hz: 44_100,
                    channel_layout: AudioChannelLayout::Stereo,
                }),
            }),
        ),
        (
            "preview.gif",
            DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
                playback: GifPlayback::Forever,
                dither: GifDither::Sierra2,
            })),
        ),
        (
            "poster.tiff",
            DeliverableKind::StillImage(StillImage {
                frame: FrameSelection::Containing {
                    at: RationalTime::zero(600).unwrap(),
                },
                encoding: ImageFormat::Tiff,
            }),
        ),
    ];
    for (name, kind) in cases {
        let value = deliverable(kind, DeliverableTarget::File { name: name.into() });
        validate_name(&value, Path::new(name)).unwrap();
        assert!(validate_name(&value, Path::new("wrong.bin")).is_err());
    }

    let package = deliverable(
        hls(),
        DeliverableTarget::Package {
            name: "stream".into(),
        },
    );
    validate_name(&package, Path::new("stream")).unwrap();
    let wrong = deliverable(
        hls(),
        DeliverableTarget::File {
            name: "stream".into(),
        },
    );
    assert!(validate_name(&wrong, Path::new("stream")).is_err());
}

#[test]
fn package_destination_consumes_its_protected_subtree() {
    let value = deliverable(
        hls(),
        DeliverableTarget::Package {
            name: "stream".into(),
        },
    );
    let error = super::validate(
        &value,
        Path::new("/tmp/stream"),
        &[PathBuf::from("/tmp/stream/source.mp4")],
    )
    .unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
}

fn deliverable(kind: DeliverableKind, target: DeliverableTarget) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_extended").unwrap(),
        target,
        kind,
    }
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
