use super::*;

#[test]
fn hls_rendition_work_is_aggregated_per_deliverable() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration = RationalTime::new(12_000 * 600, 600).unwrap();
    let output = &mut project.project.render_configs[0];
    output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_hls").unwrap(),
        target: DeliverableTarget::Package {
            name: "stream".into(),
        },
        kind: DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(HlsPackage {
            segment_duration: RationalTime::new(1_200, 600).unwrap(),
            audio: Some(HlsAudio {
                source: AudioMixSource::Master,
                encoding: HlsAudioEncoding::Aac(AacEncoding {
                    bitrate_bps: 128_000,
                    sample_rate_hz: 48_000,
                    channel_layout: AudioChannelLayout::Stereo,
                }),
            }),
            renditions: renditions(),
        })),
    }];

    let codes = validation_codes(&project);
    for code in [
        "BUDGET_VIDEO_FRAMES",
        "BUDGET_PIXEL_FRAMES",
        "BUDGET_AUDIO_SAMPLES",
    ] {
        assert_code(&codes, code);
    }
}

fn renditions() -> Vec<HlsRendition> {
    (0..8)
        .map(|index| HlsRendition {
            id: HlsRenditionId::new(format!("rnd_{index}")).unwrap(),
            raster: HlsRenditionRaster {
                width: 3_840 - index * 2,
                height: 2_160,
            },
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
        })
        .collect()
}
