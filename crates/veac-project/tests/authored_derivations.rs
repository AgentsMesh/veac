use veac_project::{build_project_source, MediaDerivation, MediaType, ProjectTargetEntry};

const SOURCE: &str = include_str!("fixtures/authored_derivation.veac");
const PROXY_VIDEO: &str = r#"MediaDerivation.ProxyVideo {
            source: identifier("source"),
            source_stream: ProjectStreamSelection { global_index: 0, type_index: 0, },
            source_clock: ProjectSourceClock.Identity {
              duration: ProjectRational { numerator: 2, denominator: 5, },
            },
            width: 16,
            height: 16,
            frame_rate: ProjectRational { numerator: 10, denominator: 1, },
            crf: 28,
          }"#;

#[test]
fn authored_media_derivations_decode_every_closed_variant() {
    for (operation, media_type, expected) in cases() {
        let source = SOURCE
            .replace(PROXY_VIDEO, operation)
            .replace("MediaType.Video", media_type);
        let project = build_project_source(&source).unwrap_or_else(|error| {
            panic!("{expected} failed: {error:?}");
        });
        let target = &project.manifest.targets[0];
        let ProjectTargetEntry::MediaDerivation { operation } = &target.entry else {
            panic!("expected media derivation");
        };
        assert_eq!(operation_name(operation), expected);
    }
}

#[test]
fn authored_derivation_checked_integers_report_nested_field_paths() {
    let source = SOURCE.replace("crf: 28", "crf: 300");
    let error = build_project_source(&source).unwrap_err();
    let veac_project::ProjectAuthoringError::Decode(error) = error else {
        panic!("expected decode error");
    };
    assert_eq!(error.path, "manifest.targets[0].entry.operation.crf");
}

fn cases() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (PROXY_VIDEO, "MediaType.Video", "proxy_video"),
        (
            r#"MediaDerivation.ProxyAudio { source: identifier("source"), source_stream: ProjectStreamSelection { global_index: 1, type_index: 0, }, source_clock: ProjectSourceClock.Identity { duration: ProjectRational { numerator: 2, denominator: 5, }, }, sample_rate: 48000, channels: 2, }"#,
            "MediaType.Audio",
            "proxy_audio",
        ),
        (
            r#"MediaDerivation.Thumbnail { source: identifier("source"), source_stream: ProjectStreamSelection { global_index: 0, type_index: 0, }, at: ProjectRational { numerator: 1, denominator: 10, }, width: 16, height: 16, }"#,
            "MediaType.Image",
            "thumbnail",
        ),
        (
            r##"MediaDerivation.Waveform { source: identifier("source"), source_stream: ProjectStreamSelection { global_index: 1, type_index: 0, }, source_clock: ProjectSourceClock.Bounded { start: ProjectRational { numerator: 0, denominator: 1, }, duration: ProjectRational { numerator: 2, denominator: 5, }, }, sample_rate: 48000, width: 64, height: 16, color: "#00ff00", }"##,
            "MediaType.Image",
            "waveform",
        ),
        (
            r#"MediaDerivation.OpticalFlow { source: identifier("source"), source_stream: ProjectStreamSelection { global_index: 0, type_index: 0, }, source_clock: ProjectSourceClock.Identity { duration: ProjectRational { numerator: 2, denominator: 5, }, }, width: 32, height: 32, frame_rate: ProjectRational { numerator: 20, denominator: 1, }, method: ProjectOpticalFlowMethod.MotionCompensated, }"#,
            "MediaType.Video",
            "optical_flow",
        ),
        (
            r#"MediaDerivation.SourceSegment { source: identifier("source"), video_stream: ProjectStreamSelection { global_index: 0, type_index: 0, }, start: ProjectRational { numerator: 0, denominator: 1, }, duration: ProjectRational { numerator: 1, denominator: 5, }, width: 16, height: 16, frame_rate: ProjectRational { numerator: 10, denominator: 1, }, audio: OptionalProjectSegmentAudio.Some { value: ProjectSegmentAudio { source_stream: ProjectStreamSelection { global_index: 1, type_index: 0, }, sample_rate: 48000, channels: 2, }, }, crf: 23, }"#,
            "MediaType.Video",
            "source_segment",
        ),
    ]
}

fn operation_name(value: &MediaDerivation) -> &'static str {
    match value {
        MediaDerivation::ProxyVideo { .. } => "proxy_video",
        MediaDerivation::ProxyAudio { .. } => "proxy_audio",
        MediaDerivation::Thumbnail { .. } => "thumbnail",
        MediaDerivation::Waveform { .. } => "waveform",
        MediaDerivation::OpticalFlow { .. } => "optical_flow",
        MediaDerivation::SourceSegment { .. } => "source_segment",
    }
}

#[test]
fn derivation_output_types_are_part_of_the_typed_contract() {
    let project = build_project_source(SOURCE).unwrap();
    let ProjectTargetEntry::MediaDerivation { operation } = &project.manifest.targets[0].entry
    else {
        panic!("expected media derivation");
    };
    assert_eq!(operation.output_media_type(), MediaType::Video);
}
