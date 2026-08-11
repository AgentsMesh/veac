use super::*;
use veac_artifact::{MediaArtifactSpec, SourceClockSpec};
use veac_project::{
    InputId, ProjectOpticalFlowMethod, ProjectRational, ProjectSegmentAudio, ProjectSourceClock,
    ProjectStreamSelection,
};

#[path = "tests/exact.rs"]
mod exact;

#[test]
fn every_project_derivation_maps_to_the_closed_artifact_spec() {
    let specs = operations()
        .iter()
        .map(convert)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(matches!(specs[0], MediaArtifactSpec::ProxyVideo(_)));
    assert!(matches!(specs[1], MediaArtifactSpec::ProxyAudio(_)));
    assert!(matches!(specs[2], MediaArtifactSpec::Thumbnail(_)));
    assert!(matches!(specs[3], MediaArtifactSpec::Waveform(_)));
    assert!(matches!(specs[4], MediaArtifactSpec::OpticalFlow(_)));
    assert!(matches!(specs[5], MediaArtifactSpec::SourceSegment(_)));
    for spec in specs {
        veac_artifact::MediaArtifactRequest {
            source_identity: veac_artifact::ContentDigest::sha256(b"source"),
            producer: veac_artifact::ProducerFingerprint {
                name: "test".into(),
                version: "1".into(),
                configuration: veac_artifact::ContentDigest::sha256(b"configuration"),
            },
            spec,
        }
        .validate()
        .unwrap();
    }
}

#[test]
fn bounded_clocks_normalize_to_one_exact_timescale() {
    let operation = MediaDerivation::ProxyVideo {
        source: source(),
        source_stream: video(),
        source_clock: ProjectSourceClock::Bounded {
            start: ProjectRational::new(1, 2),
            duration: ProjectRational::new(1, 4),
        },
        width: 16,
        height: 16,
        frame_rate: ProjectRational::new(10, 1),
        crf: 28,
    };
    let MediaArtifactSpec::ProxyVideo(value) = convert(&operation).unwrap() else {
        panic!("expected proxy video");
    };
    let SourceClockSpec::Bounded { logical_range } = value.source_clock else {
        panic!("expected bounded clock");
    };
    assert_eq!(
        (logical_range.start.value, logical_range.start.timescale),
        (2, 4)
    );
    assert_eq!(
        (
            logical_range.duration.value,
            logical_range.duration.timescale
        ),
        (1, 4)
    );
}

#[test]
fn conversion_defensively_rejects_noncanonical_or_unbounded_ratios() {
    let mut operation = operations().remove(0);
    set_rate(&mut operation, ProjectRational::new(2, 2));
    assert!(convert(&operation).is_err());
    set_rate(
        &mut operation,
        ProjectRational::new(1, u64::from(u32::MAX) + 1),
    );
    assert!(convert(&operation).is_err());
}

fn set_rate(operation: &mut MediaDerivation, value: ProjectRational) {
    let MediaDerivation::ProxyVideo { frame_rate, .. } = operation else {
        unreachable!();
    };
    *frame_rate = value;
}

fn operations() -> Vec<MediaDerivation> {
    vec![
        MediaDerivation::ProxyVideo {
            source: source(),
            source_stream: video(),
            source_clock: clock(),
            width: 16,
            height: 16,
            frame_rate: rate(),
            crf: 28,
        },
        MediaDerivation::ProxyAudio {
            source: source(),
            source_stream: audio(),
            source_clock: clock(),
            sample_rate: 48000,
            channels: 2,
        },
        MediaDerivation::Thumbnail {
            source: source(),
            source_stream: video(),
            at: ProjectRational::new(1, 10),
            width: 16,
            height: 16,
        },
        MediaDerivation::Waveform {
            source: source(),
            source_stream: audio(),
            source_clock: clock(),
            sample_rate: 48000,
            width: 64,
            height: 16,
            color: "#00ff00".into(),
        },
        MediaDerivation::OpticalFlow {
            source: source(),
            source_stream: video(),
            source_clock: clock(),
            width: 32,
            height: 32,
            frame_rate: ProjectRational::new(20, 1),
            method: ProjectOpticalFlowMethod::BlockMatching,
        },
        MediaDerivation::SourceSegment {
            source: source(),
            video_stream: video(),
            start: ProjectRational::new(0, 1),
            duration: ProjectRational::new(1, 5),
            width: 16,
            height: 16,
            frame_rate: rate(),
            audio: Some(ProjectSegmentAudio {
                source_stream: audio(),
                sample_rate: 48000,
                channels: 2,
            }),
            crf: 23,
        },
    ]
}

fn source() -> InputId {
    InputId::from("source")
}
fn video() -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index: 0,
        type_index: 0,
    }
}
fn audio() -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index: 1,
        type_index: 0,
    }
}
fn clock() -> ProjectSourceClock {
    ProjectSourceClock::Identity {
        duration: ProjectRational::new(2, 5),
    }
}
fn rate() -> ProjectRational {
    ProjectRational::new(10, 1)
}
