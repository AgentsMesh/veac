use super::*;

#[test]
fn video_selection_must_match_inventory_type_indexes_and_playability() {
    for mutate in [
        |probe: &mut MediaProbeSnapshot| probe.selected_video_stream = None,
        |probe: &mut MediaProbeSnapshot| probe.selected_video_stream = Some(selection(99, 0)),
        |probe: &mut MediaProbeSnapshot| probe.selected_video_stream = Some(selection(0, 1)),
        |probe: &mut MediaProbeSnapshot| probe.selected_video_stream = Some(selection(1, 0)),
        |probe: &mut MediaProbeSnapshot| probe.streams[0].disposition.attached_picture = true,
        |probe: &mut MediaProbeSnapshot| probe.streams[0].disposition.timed_thumbnail = true,
    ] {
        let mut project = sample_project();
        mutate(probe_mut(&mut project));
        assert_probe_code(&project, "PROBE_KIND");
    }
}

#[test]
fn image_and_audio_materials_require_only_their_selected_stream_kind() {
    let mut image = sample_project();
    let material = material_mut(&mut image);
    material.kind = MaterialKind::Image;
    material.stream_intent.audio = StreamChoice::Disabled;
    material.probe.as_mut().unwrap().selected_audio_stream = None;
    assert!(!probe_codes(&image).iter().any(|code| code == "PROBE_KIND"));

    let mut image_with_audio = image.clone();
    probe_mut(&mut image_with_audio).selected_audio_stream = Some(selection(1, 0));
    assert_probe_code(&image_with_audio, "PROBE_KIND");

    let mut audio = sample_project();
    let material = material_mut(&mut audio);
    material.kind = MaterialKind::Audio;
    material.stream_intent.video = StreamChoice::Disabled;
    material.probe.as_mut().unwrap().selected_video_stream = None;
    assert!(!probe_codes(&audio).iter().any(|code| code == "PROBE_KIND"));

    for selected in [None, Some(selection(0, 0)), Some(selection(1, 1))] {
        let mut invalid = audio.clone();
        probe_mut(&mut invalid).selected_audio_stream = selected;
        assert_probe_code(&invalid, "PROBE_KIND");
    }
}

#[test]
fn non_media_material_kinds_never_accept_probe_selection() {
    for kind in [MaterialKind::Font, MaterialKind::Lut1d, MaterialKind::Lut3d] {
        let mut project = sample_project();
        material_mut(&mut project).kind = kind;
        assert_probe_code(&project, "PROBE_KIND");
    }
}

#[test]
fn authored_stream_intent_matches_disabled_and_global_index_modes() {
    let mut matching = sample_project();
    let material = material_mut(&mut matching);
    material.stream_intent.video = StreamChoice::GlobalIndex { global_index: 0 };
    material.stream_intent.audio = StreamChoice::Disabled;
    material.probe.as_mut().unwrap().selected_audio_stream = None;
    assert!(!probe_codes(&matching)
        .iter()
        .any(|code| code == "PROBE_INTENT"));

    for choice in [
        StreamChoice::GlobalIndex { global_index: 1 },
        StreamChoice::GlobalIndex { global_index: 99 },
    ] {
        let mut project = matching.clone();
        material_mut(&mut project).stream_intent.video = choice;
        assert_probe_code(&project, "PROBE_INTENT");
    }

    let mut disabled_with_selection = sample_project();
    material_mut(&mut disabled_with_selection)
        .stream_intent
        .audio = StreamChoice::Disabled;
    assert_probe_code(&disabled_with_selection, "PROBE_INTENT");
}
