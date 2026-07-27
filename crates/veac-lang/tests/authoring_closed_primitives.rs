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

    let crf = video(&project, 0);
    assert!(matches!(
        crf.video.rate_control,
        VideoRateControl::Crf { value: 18 }
    ));
    assert_eq!(crf.pass_mode, PassMode::Single);
    assert_eq!(crf.hardware, HardwareSelection::Auto);

    let bitrate = video(&project, 1);
    assert!(matches!(
        bitrate.video.rate_control,
        VideoRateControl::Bitrate {
            target_bps: 6_000_000,
            max_bps: Some(8_000_000),
            buffer_bps: Some(12_000_000)
        }
    ));
    assert_eq!(bitrate.pass_mode, PassMode::TwoPass);
    assert_eq!(bitrate.hardware, HardwareSelection::Software);

    let lossless = video(&project, 2);
    assert!(matches!(
        lossless.video.rate_control,
        VideoRateControl::Lossless
    ));
    assert_eq!(lossless.hardware, HardwareSelection::Auto);
}

#[test]
fn output_video_rejects_open_ended_modes_and_bad_frame_rates() {
    for invalid in [
        "rate-control turbo;",
        "rate-control crf;",
        "rate-control bitrate { average 1000; bogus 2; }",
        "pass-mode three-pass;",
        "hardware quantum;",
    ] {
        let source = single_output(invalid);
        assert!(compile_round_trip(&source).is_err(), "accepted `{invalid}`");
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

fn video(project: &ProjectEnvelope, index: usize) -> &VideoDeliverable {
    let deliverable = &project.project.render_configs[index].deliverables[0];
    let DeliverableKind::Video(video) = &deliverable.kind else {
        panic!("expected video output")
    };
    video
}

fn single_output(field: &str) -> String {
    format!(
        r#"{BASE}
output video invalid {{
  sequence main;
  file-name "invalid.mp4";
  encoding {{
    container mp4;
    video {{ codec h264; pixel-format yuv420p; {field} }}
    audio none;
    captions discard;
    optimize-for-streaming false;
    pass-mode single;
    hardware auto;
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
output video crf {
  sequence main; file-name "crf.mp4";
  encoding { container mp4; video { codec h264; pixel-format yuv420p; rate-control crf { value 18; } }
    audio none; captions discard; optimize-for-streaming true; pass-mode single; hardware auto; }
}
output video bitrate {
  sequence main; file-name "bitrate.mp4";
  encoding { container mp4; video { codec h264; pixel-format yuv420p;
      rate-control bitrate { target-bps 6000000; max-bps 8000000; buffer-bps 12000000; } }
    audio none; captions discard; optimize-for-streaming false; pass-mode two-pass; hardware software; }
}
output video lossless {
  sequence main; file-name "lossless.mp4";
  encoding { container mp4; video { codec h264; pixel-format yuv420p; rate-control lossless; }
    audio none; captions discard; optimize-for-streaming false; pass-mode single; hardware auto; }
}
}"#;
