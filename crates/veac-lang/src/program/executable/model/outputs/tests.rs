use crate::program::build_source;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let clip = item(
    identifier("picture"), item_enabled(), during(0s, 2s),
    source_generated(generator_solid(#285577ff)), source_timing_native());
  let layer = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default())
    .with_item(clip);
  let timeline = sequence(
    identifier("main"), "output lookup",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  let video = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(),
    video_profile_auto(), video_level_auto());
  let movie = deliverable_video(
    identifier("movie"), delivery_file("movie.mp4"),
    video_delivery(container_mp4(), video, embedded_audio_none(),
      false, pass_single(), hardware_software()));
  let output = delivery(
    identifier("default"), timeline,
    raster_settings(canvas(320px, 180px), frame_rate(30, 1), caption_discard()),
    [movie]);
  project(identifier("lookup"), project_settings(600))
    .with_sequence(timeline).entry(timeline).with_delivery(output)
}
"#;

#[test]
fn logical_output_lookup_follows_authored_delivery_identity() {
    let built = build_source(SOURCE).unwrap();
    let config = &built.envelope().project.render_configs[0];
    let movie = built
        .deliverable_by_logical_key(&config.id, "movie")
        .unwrap();
    assert_eq!(movie.target.file_name(), Some("movie.mp4"));
    assert!(built
        .deliverable_by_logical_key(&config.id, "missing")
        .is_none());
}
