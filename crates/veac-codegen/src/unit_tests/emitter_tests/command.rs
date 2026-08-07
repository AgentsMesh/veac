use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;

use super::support::{
    bindings, emit_video_command, file_identity, fixture, input_bindings, output_bindings,
    resolved, test_font_path, visual,
};

#[test]
fn emits_structured_arguments_with_global_stream_selection() {
    let plan = resolved(&fixture());
    let command = emit_video_command(&plan, &bindings(&plan)).expect("emits");
    assert_eq!(command.inputs.len(), 1);
    assert_eq!(command.maps.len(), 1);
    let video_map = command.maps[0].clone();
    assert!(video_map.starts_with('[') && video_map.ends_with(']'));
    let graph = command.filter_graph.as_ref().expect("filter graph");
    assert!(graph.contains("[0:2]trim=start=0:duration=1"));
    assert!(graph.contains(&video_map));
    assert!(!graph.contains("[0:v]"));
    let args = command.to_args();
    assert_eq!(args[0], "-y");
    assert!(args
        .windows(2)
        .any(|pair| pair[0] == "-map" && pair[1] == video_map));
    assert_eq!(args.last().unwrap(), "/tmp/output.mp4");
}

#[test]
fn font_bindings_do_not_consume_ffmpeg_input_indexes() {
    let mut project = fixture();
    project.project.materials.push(Material {
        id: MaterialId::new("med_aaa_font").unwrap(),
        kind: MaterialKind::Font,
        source: MaterialSource::File {
            uri: "font.ttf".to_owned(),
        },
        identity: Some(file_identity(&test_font_path())),
        stream_intent: StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
        probe: None,
        authorship: None,
    });
    let mut text = project.project.sequences[0].tracks[0].clips[0].clone();
    text.id = ItemId::new("itm_text").unwrap();
    text.record_range.start = RationalTime::new(600, 600).unwrap();
    text.source = ClipSource::Text {
        text: "hello".to_owned(),
        style: TextStyle {
            font: FontRef::Material {
                material_id: MaterialId::new("med_aaa_font").unwrap(),
            },
            size_pixels: 32.0,
            color: Color {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            },
            background: None,
            outline: None,
            shadow: None,
            ..TextStyle::default()
        },
    };
    text.source_mapping = None;
    text.audio = None;
    let mut text_visual = visual();
    text_visual.opacity = Animatable::constant(1.0);
    text_visual.transform.position = Animatable::constant(Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    });
    text.visual = Some(text_visual);
    project.project.sequences[0].tracks.push(Track {
        id: TrackId::new("trk_text").unwrap(),
        kind: TrackKind::Visual,
        order: 10,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: vec![text],
    });
    let plan = resolved(&project);
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    assert_eq!(command.inputs.len(), 1);
    let graph = command.filter_graph.unwrap();
    assert!(graph.contains("[0:2]trim="));
    assert!(graph.contains("subtitles=filename='data\\:application/x-ass;base64\\,"));
    assert!(graph.contains("fontsdir="));
}

#[test]
fn public_bundle_reports_missing_input_and_output_bindings_before_emission() {
    let plan = resolved(&fixture());
    let local = output_bindings(&plan);
    let error = emit_video_command(&plan, &local).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::MissingInputBinding
    );
    assert_eq!(error.diagnostics()[0].code, "INPUT_BINDING_MISSING");

    let local = input_bindings(&plan);
    let error = emit_video_command(&plan, &local).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::MissingOutputBinding
    );
    assert!(error.to_string().contains("OUTPUT_BINDING_MISSING"));
}

#[test]
fn preflight_aggregates_invalid_schema_output_ranges_and_references() {
    let mut plan = resolved(&fixture());
    let local = bindings(&plan);
    plan.header.schema = "https://invalid.test/plan".to_owned();
    plan.output.raster.as_mut().unwrap().width = 0;
    plan.output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 0,
        channels: 0,
    });
    plan.inputs.clear();
    plan.sequences[0].duration.value = 0;
    plan.sequences[0].tracks[0].clips[0]
        .record_range
        .duration
        .value = 0;
    let duplicate = plan.sequences[0].clone();
    plan.sequences.push(duplicate);
    let error = emit_video_command(&plan, &local).unwrap_err();
    let codes: Vec<_> = error
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert!(codes.contains(&"PLAN_SCHEMA_UNSUPPORTED"));
    assert!(codes.contains(&"PLAN_RASTER_INVALID"));
    assert!(codes.contains(&"PLAN_AUDIO_OUTPUT_INVALID"));
    assert!(codes.contains(&"PLAN_INPUT_MISSING"));
    assert!(codes.contains(&"PLAN_DURATION_INVALID"));
    assert!(codes.contains(&"PLAN_RANGE_INVALID"));
    assert!(codes.contains(&"PLAN_SEQUENCE_DUPLICATE"));
}

#[test]
fn preflight_rejects_a_nested_sequence_cycle() {
    let mut plan = resolved(&fixture());
    plan.sequences[0].tracks[0].clips[0].source = veac_plan::ResolvedClipSource::Sequence {
        sequence_id: plan.entry_sequence_id.clone(),
    };
    let error = emit_video_command(&plan, &bindings(&plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == "PLAN_SEQUENCE_CYCLE"));
}
