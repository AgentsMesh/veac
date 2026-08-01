use super::support::*;
use crate::{canonical::*, resolve_one, ResolvedRenderPlan};

#[test]
fn audio_file_selects_only_its_declared_track() {
    let mut value = project();
    value.project.materials.push(audio_material("med_audio"));
    let mut audio = media_clip("itm_audio", "med_audio", 0);
    audio.audio = Some(audio_properties());
    value.project.sequences[0]
        .tracks
        .push(track("trk_audio", TrackKind::Audio, 10, vec![audio]));
    set_delivery(
        &mut value,
        mp3(AudioMixSource::Track {
            track_id: TrackId::new("trk_audio").unwrap(),
        }),
    );
    value.project.render_configs[0].raster = None;

    let plan = resolve(&value);
    assert!(!resolved_track(&plan, "trk_video").state.include_in_render);
    assert!(resolved_track(&plan, "trk_audio").state.audio_enabled);
}

#[test]
fn animated_and_still_deliveries_request_visual_without_audio() {
    for delivery in [gif(), still()] {
        let mut value = audiovisual_project();
        set_delivery(&mut value, delivery);
        let plan = resolve(&value);
        let track = resolved_track(&plan, "trk_video");
        assert!(track.state.visual_enabled);
        assert!(!track.state.audio_enabled);
    }
}

#[test]
fn hls_audio_demand_follows_the_package_contract() {
    for (audio, expected) in [
        (None, false),
        (Some(hls_audio(AudioMixSource::Master)), true),
    ] {
        let mut value = audiovisual_project();
        set_delivery(&mut value, hls(audio));
        let plan = resolve(&value);
        let track = resolved_track(&plan, "trk_video");
        assert!(track.state.visual_enabled);
        assert_eq!(track.state.audio_enabled, expected);
    }
}

#[test]
fn hls_audio_selects_only_its_declared_track() {
    let mut value = project();
    value.project.materials.push(audio_material("med_audio"));
    let mut audio = media_clip("itm_audio", "med_audio", 0);
    audio.audio = Some(audio_properties());
    value.project.sequences[0]
        .tracks
        .push(track("trk_audio", TrackKind::Audio, 10, vec![audio]));
    set_delivery(
        &mut value,
        hls(Some(hls_audio(AudioMixSource::Track {
            track_id: TrackId::new("trk_audio").unwrap(),
        }))),
    );

    let plan = resolve(&value);
    assert!(!resolved_track(&plan, "trk_video").state.audio_enabled);
    assert!(resolved_track(&plan, "trk_audio").state.audio_enabled);
}

fn audiovisual_project() -> ProjectEnvelope {
    let mut value = project();
    let clip = &mut value.project.sequences[0].tracks[0].clips[0];
    clip.visual = Some(visual_properties());
    clip.audio = Some(audio_properties());
    value
}

fn set_delivery(value: &mut ProjectEnvelope, delivery: Deliverable) {
    value.project.render_configs[0].deliverables = vec![delivery];
}

fn resolve(value: &ProjectEnvelope) -> ResolvedRenderPlan {
    resolve_one(value, &RenderConfigId::new("out_main").unwrap()).unwrap()
}

fn resolved_track<'a>(plan: &'a ResolvedRenderPlan, id: &str) -> &'a crate::ResolvedTrack {
    plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == id)
        .unwrap()
}

fn mp3(source: AudioMixSource) -> Deliverable {
    file(
        "podcast.mp3",
        DeliverableKind::AudioFile(AudioFile {
            source,
            encoding: AudioFileEncoding::Mp3(Mp3Encoding {
                bitrate_bps: 192_000,
                sample_rate_hz: 48_000,
                channel_layout: AudioChannelLayout::Stereo,
            }),
        }),
    )
}

fn gif() -> Deliverable {
    file(
        "preview.gif",
        DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
            playback: GifPlayback::Forever,
            dither: GifDither::Sierra2,
        })),
    )
}

fn still() -> Deliverable {
    file(
        "cover.png",
        DeliverableKind::StillImage(StillImage {
            frame: FrameSelection::Containing { at: time(0) },
            encoding: ImageFormat::Png,
        }),
    )
}

fn hls(audio: Option<HlsAudio>) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_format").unwrap(),
        target: DeliverableTarget::Package {
            name: "stream".into(),
        },
        kind: DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(HlsPackage {
            segment_duration: time(600),
            audio,
            renditions: vec![HlsRendition {
                id: HlsRenditionId::new("rnd_main").unwrap(),
                raster: HlsRenditionRaster {
                    width: 1280,
                    height: 720,
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
            }],
        })),
    }
}

fn file(name: &str, kind: DeliverableKind) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_format").unwrap(),
        target: DeliverableTarget::File { name: name.into() },
        kind,
    }
}

fn hls_audio(source: AudioMixSource) -> HlsAudio {
    HlsAudio {
        source,
        encoding: HlsAudioEncoding::Aac(AacEncoding {
            bitrate_bps: 128_000,
            sample_rate_hz: 48_000,
            channel_layout: AudioChannelLayout::Stereo,
        }),
    }
}
