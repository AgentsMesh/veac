use veac_ir::{
    DeliverableKind, HardwareSelection, PassMode, ProjectEnvelope, Rational, VideoDeliverable,
    VideoRateControl,
};
use veac_lang::{format_document, lower_document, parse};

#[test]
fn output_video_closed_modes_survive_public_round_trip() {
    let project = compile_round_trip(OUTPUTS).unwrap();
    assert_eq!(
        project.project.sequences[0].settings.frame_rate,
        Rational::new(24_000, 1_001).unwrap()
    );

    let crf = video(&project, "dlv_crf");
    assert!(matches!(
        crf.video.rate_control,
        VideoRateControl::Crf { value: 18 }
    ));
    assert_eq!(crf.pass_mode, PassMode::Single);
    assert_eq!(crf.hardware, HardwareSelection::Auto);

    let bitrate = video(&project, "dlv_bitrate");
    assert!(matches!(
        bitrate.video.rate_control,
        VideoRateControl::Bitrate {
            target_bps: 6_000_000,
            max_bps: Some(8_000_000),
            buffer_size_bits: Some(12_000_000)
        }
    ));
    assert_eq!(bitrate.pass_mode, PassMode::TwoPass);
    assert_eq!(bitrate.hardware, HardwareSelection::Software);

    let lossless = video(&project, "dlv_lossless");
    assert!(matches!(
        lossless.video.rate_control,
        VideoRateControl::Lossless
    ));
    assert_eq!(lossless.hardware, HardwareSelection::Auto);
}

#[test]
fn output_video_rejects_open_ended_modes_and_bad_frame_rates() {
    for (video_field, mux_field) in [
        ("rate-control turbo;", ""),
        ("rate-control crf;", ""),
        ("rate-control capped { target 1mbps; bogus 2; }", ""),
        ("", "passes three-pass;"),
        ("", "accelerator quantum;"),
    ] {
        let source = single_output(video_field, mux_field);
        assert!(
            compile_round_trip(&source).is_err(),
            "accepted `{video_field} {mux_field}`"
        );
    }

    let zero_rate = OUTPUTS.replace("24000/1001fps", "0/1001fps");
    assert!(compile_round_trip(&zero_rate).is_err());
}

fn compile_round_trip(source: &str) -> Result<ProjectEnvelope, String> {
    let document = parse(source).map_err(|error| format!("{error:?}"))?;
    let formatted = format_document(&document);
    let reparsed = parse(&formatted).map_err(|error| format!("{error:?}"))?;
    lower_document(&reparsed).map_err(|error| format!("{error:?}"))
}

fn video<'a>(project: &'a ProjectEnvelope, id: &str) -> &'a VideoDeliverable {
    let deliverable = project.project.render_configs[0]
        .deliverables
        .iter()
        .find(|value| value.id.as_str() == id)
        .unwrap_or_else(|| panic!("missing video artifact {id}"));
    let DeliverableKind::Video(video) = &deliverable.kind else {
        panic!("expected video output")
    };
    video
}

fn single_output(video_field: &str, mux_field: &str) -> String {
    format!(
        r#"{BASE}
delivery invalid {{
  sequence main;
  raster {{ canvas 1920px by 1080px; frame-rate 24000/1001fps; captions discard; }}
  artifact video invalid {{
    target file "invalid.mp4";
    mux mp4 {{
      layout standard;
      video h264 {{ pixel-format yuv420p; {video_field} }}
      audio none;
      {mux_field}
    }}
  }}
}}
}}"#
    )
}

const BASE: &str = r#"project closed-output {
settings {
  timebase 1/600;
  canvas 1920px by 1080px;
  frame-rate 24000/1001fps;
  sample-rate 48000hz;
}
entry sequence main;
sequence main {}
"#;

const OUTPUTS: &str = r#"project closed-output {
settings {
  timebase 1/600;
  canvas 1920px by 1080px;
  frame-rate 24000/1001fps;
  sample-rate 48000hz;
}
entry sequence main;
sequence main {}
delivery closed-modes {
  sequence main;
  raster { canvas 1920px by 1080px; frame-rate 24000/1001fps; captions discard; }
  artifact video crf {
    target file "crf.mp4";
    mux mp4 { layout fast-start; video h264 { pixel-format yuv420p; rate-control crf { value 18; } }
      audio none; passes single; accelerator auto; }
  }
  artifact video bitrate {
    target file "bitrate.mp4";
    mux mp4 { layout standard; video h264 { pixel-format yuv420p;
        rate-control capped { target 6mbps; max 8mbps; buffer 12mbit; } }
      audio none; passes two-pass; accelerator software; }
  }
  artifact video lossless {
    target file "lossless.mp4";
    mux mp4 { layout standard; video h264 { pixel-format yuv420p; rate-control lossless; }
      audio none; passes single; accelerator auto; }
  }
}
}"#;
