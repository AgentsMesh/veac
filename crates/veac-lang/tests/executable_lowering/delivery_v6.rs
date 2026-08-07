use veac_ir::{
    AdaptivePackage, AudioCodec, DeliverableKind, DeliverableTarget, HlsAudioEncoding,
    HlsVideoEncoding, OutputFormat, VideoCodec,
};

use super::support;

#[path = "delivery_v6/coverage.rs"]
mod coverage;
#[path = "delivery_v6/variants.rs"]
mod variants;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let picture = item(
    identifier("picture"), item_enabled(), during(0s, 6s),
    source_generated(generator_solid(#285577ff)), source_timing_native()
  );
  let visual = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default()
  ).with_item(picture);
  let captions = caption_layer(
    identifier("captions"), 1, placement_free(), state, track_routing_default()
  );
  let timeline = sequence(
    identifier("main"), "交付能力验证",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  ).with_layer(visual).with_layer(captions);
  let video = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(20), gop_auto(), b_frames_auto(),
    video_profile_present(profile_h264_high()), video_level_present(h264_level(4, 1))
  );
  let movie = deliverable_video(
    identifier("movie"), delivery_file("main.mp4"),
    video_delivery(container_mp4(), video,
      embedded_audio_present(audio_output(audio_aac(), 48000, 2)),
      true, pass_single(), hardware_software())
  );
  let frames = deliverable_image_frames(
    identifier("frames"), delivery_image_sequence("frame-%04d.png"), image_png(), 1
  );
  let sidecar = deliverable_caption_sidecar(
    identifier("captions"), delivery_file("captions.srt"), caption_srt(), [captions]
  );
  let stem = deliverable_audio_stem(
    identifier("stem"), delivery_file("master.wav"), stem_wav(),
    audio_output(audio_pcm_s24le(), 48000, 2), mix_master()
  );
  let mp3 = deliverable_mp3(
    identifier("mp3"), delivery_file("master.mp3"), mix_master(), 192000, 48000,
    channel_stereo()
  );
  let scope = deliverable_scope(
    identifier("scope"), delivery_file("scope.png"), scope_waveform(), 1s,
    canvas(320px, 180px), image_png()
  );
  let gif = deliverable_gif(
    identifier("gif"), delivery_file("preview.gif"), gif_forever(), gif_dither_sierra2()
  );
  let still = deliverable_still(
    identifier("still"), delivery_file("still.png"), 2s, image_png()
  );
  let rendition = hls_rendition(
    identifier("r360p"), canvas(640px, 360px), 800000, 1000000, 2000000,
    hls_profile_present(hls_profile_high()), video_level_present(h264_level(3, 1)),
    video_color_unspecified(), b_frames_count(2)
  );
  let hls = deliverable_hls(
    identifier("hls"), delivery_package("stream"), 2s,
    hls_audio_aac(mix_master(), 128000, 48000, channel_stereo()), [rendition]
  );
  let output = delivery(
    identifier("default"), timeline,
    raster_settings(canvas(640px, 360px), frame_rate(30, 1), caption_discard()),
    [movie, frames, sidecar, stem, mp3, scope, gif, still, hls]
  );
  project(identifier("delivery"), project_settings(600))
    .with_sequence(timeline).entry(timeline).with_delivery(output)
}
"#;

#[test]
fn all_delivery_artifact_families_lower_to_canonical_ir() {
    let envelope = support::envelope(SOURCE);
    let output = &envelope.project.render_configs[0];
    assert_eq!(output.deliverables.len(), 9);
    assert_eq!(output.raster.as_ref().unwrap().width, 640);
    assert!(output
        .deliverables
        .windows(2)
        .all(|pair| pair[0].id < pair[1].id));
    assert!(output.deliverables.iter().any(|artifact| matches!(
        artifact.kind,
        DeliverableKind::Video(ref video)
            if video.container == OutputFormat::Mp4 && video.video.codec == VideoCodec::H264
    )));
    assert!(output.deliverables.iter().any(|artifact| matches!(
        (&artifact.target, &artifact.kind),
        (
            DeliverableTarget::Package { .. },
            DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(_))
        )
    )));
    let provenance = &envelope.project.authorship.as_ref().unwrap().deliveries;
    assert_eq!(provenance.len(), 1);
    let entry = provenance
        .iter()
        .find(|entry| entry.render_config_id == output.id)
        .unwrap();
    assert_eq!(
        entry.entity.logical_path.last().unwrap().as_str(),
        "default"
    );
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn typed_authorship_paths_disambiguate_sibling_entity_kinds() {
    let source = SOURCE.replace(
        "identifier(\"default\"), timeline",
        "identifier(\"main\"), timeline",
    );
    let envelope = support::envelope(&source);
    let project = envelope.project.authorship.as_ref().unwrap();
    let sequence = match envelope.project.sequences[0].authorship.as_ref().unwrap() {
        veac_ir::SequenceAuthorship::Veac { entity, .. } => entity,
        veac_ir::SequenceAuthorship::Otio { .. } => panic!("expected VEAC authorship"),
    };
    let delivery = &project.deliveries[0].entity;
    assert_eq!(sequence.logical_path[1].as_str(), "sequence");
    assert_eq!(delivery.logical_path[1].as_str(), "delivery");
    assert_ne!(sequence.logical_path, delivery.logical_path);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn hls_and_audio_encodings_remain_closed_typed_values() {
    let envelope = support::envelope(SOURCE);
    let artifacts = &envelope.project.render_configs[0].deliverables;
    let hls = artifacts
        .iter()
        .find_map(|artifact| match &artifact.kind {
            DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(value)) => Some(value),
            _ => None,
        })
        .unwrap();
    assert!(matches!(
        hls.audio.as_ref().unwrap().encoding,
        HlsAudioEncoding::Aac(_)
    ));
    assert!(matches!(
        hls.renditions[0].encoding,
        HlsVideoEncoding::H264(_)
    ));
    let video = artifacts
        .iter()
        .find_map(|artifact| match &artifact.kind {
            DeliverableKind::Video(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(video.audio.as_ref().unwrap().codec, AudioCodec::Aac);
}
