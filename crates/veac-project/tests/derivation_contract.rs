mod support;

use support::derivation::{configure, operations};
use support::{assert_code, manifest};
use veac_project::*;

#[test]
fn every_media_derivation_has_a_fully_parameterized_valid_contract() {
    for operation in operations() {
        let mut project = manifest();
        configure(&mut project.targets[0], operation);
        validate_manifest(&project).unwrap();
    }
}

#[test]
fn derivations_reject_unknown_or_non_media_sources_and_wrong_outputs() {
    let mut missing = manifest();
    configure(&mut missing.targets[0], operations().remove(0));
    missing.targets[0].inputs[0].id = InputId::from("other");
    assert_code(&missing, IssueCode::UnknownReference);

    let mut literal = manifest();
    configure(&mut literal.targets[0], operations().remove(0));
    literal.targets[0].inputs[0].source = ProjectInputSource::Literal {
        value: ProjectLiteral::Text {
            value: "source".into(),
        },
    };
    assert_code(&literal, IssueCode::InvalidAction);

    let mut output = manifest();
    configure(&mut output.targets[0], operations().remove(1));
    output.targets[0].outputs[0] = ProjectOutput::Media {
        id: OutputId::from("video"),
        media_type: MediaType::Video,
    };
    assert_code(&output, IssueCode::InvalidAction);
}

#[test]
fn derivation_parameters_fail_closed_before_execution() {
    let invalid = [
        MediaDerivation::ProxyVideo {
            source: source(),
            source_stream: video_stream(),
            source_clock: identity(ProjectRational::new(0, 1)),
            width: 0,
            height: 16,
            frame_rate: ProjectRational::new(0, 1),
            crf: 52,
        },
        MediaDerivation::ProxyAudio {
            source: source(),
            source_stream: audio_stream(),
            source_clock: identity(duration()),
            sample_rate: 0,
            channels: 9,
        },
        MediaDerivation::Thumbnail {
            source: source(),
            source_stream: video_stream(),
            at: ProjectRational::new(1, 3),
            width: 16,
            height: 0,
        },
        MediaDerivation::Waveform {
            source: source(),
            source_stream: audio_stream(),
            source_clock: identity(duration()),
            sample_rate: 0,
            width: 0,
            height: 16,
            color: "red;blue".into(),
        },
        MediaDerivation::OpticalFlow {
            source: source(),
            source_stream: video_stream(),
            source_clock: ProjectSourceClock::Bounded {
                start: ProjectRational::new(1, 3),
                duration: duration(),
            },
            width: 16,
            height: 16,
            frame_rate: ProjectRational::new(-1, 1),
            method: ProjectOpticalFlowMethod::BlockMatching,
        },
        MediaDerivation::SourceSegment {
            source: source(),
            video_stream: video_stream(),
            start: ProjectRational::new(-1, 10),
            duration: ProjectRational::new(0, 1),
            width: 0,
            height: 16,
            frame_rate: ProjectRational::new(10, 1),
            audio: Some(ProjectSegmentAudio {
                source_stream: audio_stream(),
                sample_rate: 0,
                channels: 0,
            }),
            crf: 52,
        },
    ];
    for operation in invalid {
        let mut project = manifest();
        configure(&mut project.targets[0], operation);
        assert_code(&project, IssueCode::InvalidAction);
    }
}

fn source() -> InputId {
    InputId::from("source")
}
fn duration() -> ProjectRational {
    ProjectRational::new(2, 5)
}
fn identity(duration: ProjectRational) -> ProjectSourceClock {
    ProjectSourceClock::Identity { duration }
}
fn video_stream() -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index: 0,
        type_index: 0,
    }
}
fn audio_stream() -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index: 1,
        type_index: 0,
    }
}
