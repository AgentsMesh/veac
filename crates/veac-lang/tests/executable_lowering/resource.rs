use veac_ir::{
    ClipSource, FrameSynthesisPolicy, HashAlgorithm, MaterialKind, MaterialSource,
    PlaybackDirection, SourceOutOfRangePolicy, SourceTimeMap, StreamChoice,
};

use super::support;

#[path = "resource/coverage.rs"]
mod coverage;

const SOURCE: &str = r#"
fn main(context: Context) -> Project {
    let hero = image_resource(identifier("hero"), resource_file("assets/hero.png"),
        sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    let clip = item(identifier("hero"), item_enabled(), during(0s, 2s),
        source_media(hero), source_timing_native());
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
        track_routing_default()).with_item(clip);
    let timeline = sequence(identifier("main"), "本地图片",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(visual);
    project(identifier("demo"), project_settings(600)).with_resource(hero)
        .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn local_image_lowers_to_canonical_material_and_media_mapping() {
    let envelope = support::envelope(SOURCE);
    let project = &envelope.project;
    assert_eq!(project.materials.len(), 1);
    let material = &project.materials[0];
    assert!(material.id.as_str().starts_with("med_"));
    assert_eq!(material.kind, MaterialKind::Image);
    assert_eq!(
        material.source,
        MaterialSource::File {
            uri: "assets/hero.png".to_owned()
        }
    );
    let identity = material.identity.as_ref().expect("authored identity");
    assert_eq!(identity.algorithm, HashAlgorithm::Sha256);
    assert_eq!(identity.digest, "a".repeat(64));
    assert_eq!(material.stream_intent.video, StreamChoice::Auto);
    assert_eq!(material.stream_intent.audio, StreamChoice::Disabled);
    assert_eq!(material.probe, None);
    assert_eq!(
        material
            .authorship
            .as_ref()
            .unwrap()
            .logical_path
            .iter()
            .map(|part| part.as_str())
            .collect::<Vec<_>>(),
        vec!["demo", "resource", "hero"]
    );

    let clip = support::clip(&envelope, 0);
    assert_eq!(
        clip.source,
        ClipSource::Media {
            material_id: material.id.clone()
        }
    );
    let mapping = clip.source_mapping.as_ref().expect("media mapping");
    let SourceTimeMap::Linear {
        source_start,
        rate,
        repeat,
        direction,
    } = mapping.time_map
    else {
        panic!("expected linear source mapping")
    };
    assert_eq!((source_start.value, source_start.timescale), (0, 600));
    assert_eq!((rate.numerator, rate.denominator), (1, 1));
    assert_eq!(repeat, 1);
    assert_eq!(direction, PlaybackDirection::Forward);
    assert_eq!(mapping.frame_synthesis, FrameSynthesisPolicy::Nearest);
    assert_eq!(mapping.out_of_range, SourceOutOfRangePolicy::Strict);
    assert!(clip.visual.is_some());
    assert!(clip.audio.is_none());
    assert!(veac_ir::validate(&envelope).is_ok());
    let json = veac_ir::canonical_json(&envelope).unwrap();
    assert_eq!(veac_ir::decode_canonical_json(&json).unwrap(), envelope);
}

#[test]
fn heterogeneous_attachment_order_does_not_rename_materials() {
    let resource_first = support::envelope(&ordered_project(true, "demo"));
    let sequence_first = support::envelope(&ordered_project(false, "demo"));
    assert_eq!(
        resource_first.project.materials[0].id,
        sequence_first.project.materials[0].id
    );
    assert_eq!(
        resource_first.project.sequences[0].tracks[0].clips[0].source,
        sequence_first.project.sequences[0].tracks[0].clips[0].source
    );
}

#[test]
fn equal_resource_keys_in_different_projects_have_distinct_ids() {
    let first = support::envelope(&ordered_project(true, "first"));
    let second = support::envelope(&ordered_project(true, "second"));
    assert_ne!(
        first.project.materials[0].id,
        second.project.materials[0].id
    );
}

#[test]
fn non_canonical_project_relative_uris_fail_closed() {
    for path in [
        "",
        "/absolute.png",
        "../escape.png",
        "a/../b.png",
        "a//b.png",
        "./a.png",
        "C:/drive.png",
        r"a\b.png",
    ] {
        let source = resource_only_project(path);
        let error = support::error(&source);
        assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR", "{path}");
        assert!(
            error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"),
            "{path}: {}",
            error.message
        );
    }
}

fn ordered_project(resource_first: bool, key: &str) -> String {
    let resource = ".with_resource(hero)";
    let sequence = ".with_sequence(timeline)";
    let composition = if resource_first {
        format!("{resource}{sequence}")
    } else {
        format!("{sequence}{resource}")
    };
    format!(
        r#"fn main(context: Context) -> Project {{
          let hero = image_resource(identifier("hero"), resource_file("assets/hero.png"),
            sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
          let clip = item(identifier("hero"), item_enabled(), during(0s, 1s),
            source_media(hero), source_timing_native());
          let state = track_state(track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked());
          let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
            track_routing_default()).with_item(clip);
          let timeline = sequence(identifier("main"), "资源顺序",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
            .with_layer(visual);
          project(identifier("{key}"), project_settings(600))
            {composition}.entry(timeline)
        }}"#
    )
}

fn resource_only_project(path: &str) -> String {
    let path = path.replace('\\', "\\\\");
    format!(
        r#"fn main(context: Context) -> Project {{
          let hero = image_resource(identifier("hero"), resource_file("{path}"),
              sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
          let timeline = sequence(identifier("main"), "资源路径",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
          project(identifier("demo"), project_settings(600)).with_resource(hero)
            .with_sequence(timeline).entry(timeline)
        }}"#
    )
}
