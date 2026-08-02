use super::*;

#[test]
fn mp3_contract_matches_libmp3lame_rates_and_layouts() {
    assert_valid(mp3(48_000, 192_000, AudioChannelLayout::Stereo));
    assert_valid(mp3(11_025, 64_000, AudioChannelLayout::Mono));
    for (sample_rate, bitrate) in [
        (96_000, 192_000),
        (10_000, 64_000),
        (8_000, 320_000),
        (48_000, 8_000),
        (48_000, 9_000),
    ] {
        assert_invalid(
            mp3(sample_rate, bitrate, AudioChannelLayout::Stereo),
            "OUTPUT_AUDIO_FILE",
        );
    }
}

#[test]
fn visual_delivery_formats_and_targets_are_explicit() {
    assert_valid(deliverable(
        "preview.gif",
        DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
            playback: GifPlayback::Forever,
            dither: GifDither::Sierra2,
        })),
    ));
    assert_valid(deliverable(
        "cover.png",
        DeliverableKind::StillImage(StillImage {
            frame: FrameSelection::Containing {
                at: RationalTime::zero(600).unwrap(),
            },
            encoding: ImageFormat::Png,
        }),
    ));
    assert_invalid(
        deliverable(
            "cover.jpg",
            DeliverableKind::StillImage(StillImage {
                frame: FrameSelection::Containing {
                    at: RationalTime::zero(600).unwrap(),
                },
                encoding: ImageFormat::Png,
            }),
        ),
        "OUTPUT_STILL_IMAGE",
    );
}

#[test]
fn hls_requires_package_target_and_even_yuv420_renditions() {
    assert_valid(hls(1280, 720));
    assert_invalid(hls(641, 360), "OUTPUT_HLS_PACKAGE");
    let mut wrong_target = hls(640, 360);
    wrong_target.target = DeliverableTarget::File {
        name: "stream.m3u8".into(),
    };
    assert_invalid(wrong_target, "OUTPUT_TARGET_KIND");
}

#[test]
fn hls_segment_color_and_audio_source_contracts_are_executable() {
    let mut subsecond = hls(640, 360);
    let DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) = &mut subsecond.kind
    else {
        unreachable!()
    };
    settings.segment_duration = RationalTime::new(599, 600).unwrap();
    assert_invalid(subsecond, "OUTPUT_HLS_PACKAGE");

    let mut longer_than_timeline = hls(640, 360);
    let DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) =
        &mut longer_than_timeline.kind
    else {
        unreachable!()
    };
    settings.segment_duration = RationalTime::new(1_200, 600).unwrap();
    assert_valid(longer_than_timeline);

    let mut excessive = hls(640, 360);
    let DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) = &mut excessive.kind
    else {
        unreachable!()
    };
    settings.segment_duration = RationalTime::new(60 * 600 + 1, 600).unwrap();
    assert_invalid(excessive, "OUTPUT_HLS_PACKAGE");

    let mut hdr_h264 = hls(640, 360);
    let DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) = &mut hdr_h264.kind
    else {
        unreachable!()
    };
    let HlsVideoEncoding::H264(encoding) = &mut settings.renditions[0].encoding;
    encoding.color_space = Some(ColorSpace {
        primaries: ColorPrimaries::Bt2020,
        transfer: ColorTransfer::Smpte2084,
        matrix: ColorMatrix::Bt2020Ncl,
        range: ColorRange::Limited,
    });
    assert_invalid(hdr_h264, "OUTPUT_HLS_PACKAGE");

    let mut baseline_b_frames = hls(640, 360);
    let DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) =
        &mut baseline_b_frames.kind
    else {
        unreachable!()
    };
    let HlsVideoEncoding::H264(encoding) = &mut settings.renditions[0].encoding;
    encoding.profile = Some(HlsH264Profile::Baseline);
    encoding.b_frames = Some(1);
    assert_invalid(baseline_b_frames, "OUTPUT_HLS_PACKAGE");
}

#[test]
fn mp3_bus_source_ignores_non_audio_track_kinds() {
    let bus_id = BusId::new("bus_caption").unwrap();
    let mut project = project_with(mp3(48_000, 192_000, AudioChannelLayout::Stereo));
    project.project.sequences[0].tracks[1].routing = TrackRouting::AudioBus {
        bus_id: bus_id.clone(),
    };
    let DeliverableKind::AudioFile(settings) =
        &mut project.project.render_configs[0].deliverables[0].kind
    else {
        unreachable!()
    };
    settings.source = AudioMixSource::Bus { bus_id };
    assert_code(&validation_codes(&project), "OUTPUT_AUDIO_FILE");
}

fn mp3(sample_rate_hz: u32, bitrate_bps: u32, channel_layout: AudioChannelLayout) -> Deliverable {
    deliverable(
        "podcast.mp3",
        DeliverableKind::AudioFile(AudioFile {
            source: AudioMixSource::Master,
            encoding: AudioFileEncoding::Mp3(Mp3Encoding {
                bitrate_bps,
                sample_rate_hz,
                channel_layout,
            }),
        }),
    )
}

fn hls(width: u32, height: u32) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_aux").unwrap(),
        target: DeliverableTarget::Package {
            name: "stream".into(),
        },
        kind: DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(HlsPackage {
            segment_duration: RationalTime::new(600, 600).unwrap(),
            audio: Some(HlsAudio {
                source: AudioMixSource::Master,
                encoding: HlsAudioEncoding::Aac(AacEncoding {
                    bitrate_bps: 128_000,
                    sample_rate_hz: 48_000,
                    channel_layout: AudioChannelLayout::Stereo,
                }),
            }),
            renditions: vec![HlsRendition {
                id: HlsRenditionId::new("rnd_main").unwrap(),
                raster: HlsRenditionRaster { width, height },
                encoding: HlsVideoEncoding::H264(HlsH264Encoding {
                    rate_control: HlsCappedBitrate {
                        target_bps: 3_000_000,
                        max_bps: 3_210_000,
                        buffer_size_bits: 6_000_000,
                    },
                    profile: None,
                    level: None,
                    color_space: None,
                    b_frames: None,
                }),
            }],
        })),
    }
}
