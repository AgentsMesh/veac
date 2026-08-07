use veac_ir::ClipSource;
use veac_lang::program::build_source;

#[test]
fn timeline_template_and_multicam_controls_reach_canonical_ir() {
    let built = build_source(PROJECT).unwrap();
    let envelope = built.envelope();
    let main = envelope
        .project
        .sequences
        .iter()
        .find(|sequence| sequence.id == envelope.project.entry_sequence_id)
        .unwrap();
    let track = &main.tracks[0];
    assert!(!track.state.enabled && track.state.muted && track.state.solo && track.state.locked);
    assert!(!track.clips[0].enabled);
    assert!(track.clips[0].replaceable.is_some());
    assert!(matches!(track.clips[0].source, ClipSource::Media { .. }));
    assert!(matches!(track.clips[1].source, ClipSource::Sequence { .. }));
    assert!(matches!(track.clips[2].source, ClipSource::Multicam { .. }));
    veac_ir::validate(envelope).unwrap();
}

#[test]
fn typed_references_reject_values_from_the_wrong_owner() {
    for (old, new) in [
        ("source_media(hero)", "source_media(nested)"),
        (
            "source_nested_sequence(nested)",
            "source_nested_sequence(hero)",
        ),
        (
            "multicam_sync(multicam_sync_manual(), host_angle)",
            "multicam_sync(multicam_sync_manual(), hero)",
        ),
        (
            "multicam_angle(identifier(\"host\"), camera, 0s)",
            "multicam_angle(identifier(\"host\"), context, 0s)",
        ),
        ("source_multicam(group, [", "source_multicam(hero, ["),
        (
            "multicam_switch(host_angle, during(0s, 1s))",
            "multicam_switch(hero, during(0s, 1s))",
        ),
    ] {
        let error = build_source(&PROJECT.replacen(old, new, 1)).unwrap_err();
        let diagnostic = &error.as_slice()[0];
        assert_eq!(diagnostic.code, "PROGRAM_FUNCTION_EXPRESSION");
        assert!(
            diagnostic.message.contains("EXPRESSION_CALL_ARGUMENT_TYPE"),
            "wrong owner `{new}` was not rejected as a typed reference: {error}"
        );
    }
}

const PROJECT: &str = r#"
fn main(context: Context) -> Project {
  let hero = video_resource(identifier("hero"), resource_file("hero.mov"),
    sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    stream_intent(stream_auto(), stream_disabled()));
  let camera = video_resource(identifier("camera"), resource_file("camera.mov"),
    sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    stream_intent(stream_auto(), stream_disabled()));
  let guest = video_resource(identifier("guest"), resource_file("guest.mov"),
    sha256("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"),
    stream_intent(stream_auto(), stream_disabled()));
  let host_angle = multicam_angle(identifier("host"), camera, 0s);
  let guest_angle = multicam_angle(identifier("guest"), guest, 0s);
  let group = multicam_group(identifier("interview"),
    multicam_sync(multicam_sync_manual(), host_angle), [host_angle, guest_angle]);

  let nested_card = item(identifier("card"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#245b78ff)), source_timing_native());
  let normal = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let nested_layer = visual_layer(identifier("nested"), 0, placement_free(), normal,
    track_routing_default()).with_item(nested_card);
  let nested = sequence(identifier("nested"), "嵌套时间线",
    sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000))
    .with_layer(nested_layer);

  let media = item(identifier("media"), item_disabled(), during(0s, 2s),
    source_media(hero), source_timing_native()).with_template(template_contract(
      slot_video(), fill_fit_duration(), "主视频替换位",
      source_duration_at_least(1s), template_text_locked()));
  let nested_item = item(identifier("nested-item"), item_enabled(), during(2s, 1s),
    source_nested_sequence(nested), source_timing_native());
  let cut = item(identifier("cut"), item_enabled(), during(3s, 1s),
    source_multicam(group, [
      multicam_switch(host_angle, during(0s, 1s))
    ]), source_timing_native());
  let disabled = track_state(track_playback_disabled(), track_audio_muted(),
    track_isolation_solo(), track_editing_locked());
  let program = video_layer(identifier("program"), 0, placement_free(), disabled,
    track_routing_default()).with_item(media).with_item(nested_item).with_item(cut);
  let main = sequence(identifier("main"), "主时间线",
    sequence_settings(canvas(1280px, 720px), frame_rate(30, 1), 48000))
    .with_layer(program);
  project(identifier("timeline-controls"), project_settings(600))
    .with_resource(hero).with_resource(camera).with_resource(guest)
    .with_multicam_group(group).with_sequence(nested).with_sequence(main).entry(main)
}
"#;
