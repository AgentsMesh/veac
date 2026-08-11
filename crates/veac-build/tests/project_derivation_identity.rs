use std::collections::{BTreeMap, BTreeSet};

use veac_build::{
    BuildAction, GraphBuilder, NodeId, NodeSpec, OutputSlot, ProjectAction, ProjectComputation,
};
use veac_project::{
    InputId, MediaDerivation, ProjectOpticalFlowMethod, ProjectRational, ProjectSegmentAudio,
    ProjectSourceClock, ProjectStreamSelection, TargetId, TargetInstanceId,
};

#[test]
fn every_derivation_parameter_is_part_of_action_and_graph_identity() {
    let mut graph_digests = BTreeSet::new();
    for operation in operations() {
        let action = ProjectAction::MediaDerivation {
            computation: computation(),
            operation: operation.clone(),
        };
        let canonical: serde_json::Value =
            serde_json::from_slice(&action.canonical_bytes().unwrap()).unwrap();
        assert_eq!(
            canonical["operation"],
            serde_json::to_value(&operation).unwrap()
        );

        let mut builder = GraphBuilder::new();
        let output = OutputSlot::<Media>::new("output").unwrap();
        builder
            .add_node(NodeSpec::new(NodeId::new("derive").unwrap(), action).output(&output))
            .unwrap();
        graph_digests.insert(builder.validate().unwrap().digest().value.clone());
    }
    assert_eq!(graph_digests.len(), 6);
}

struct Media;

fn computation() -> ProjectComputation {
    ProjectComputation {
        instance: TargetInstanceId::from("derive"),
        target: TargetId::from("derive"),
        profile: None,
        locale: None,
        matrix: BTreeMap::new(),
        inputs: Vec::new(),
        outputs: Vec::new(),
        bound_sources: Vec::new(),
    }
}

fn operations() -> Vec<MediaDerivation> {
    vec![
        MediaDerivation::ProxyVideo {
            source: source(),
            source_stream: stream(2, 1),
            source_clock: bounded(),
            width: 640,
            height: 360,
            frame_rate: rational(30, 1),
            crf: 21,
        },
        MediaDerivation::ProxyAudio {
            source: source(),
            source_stream: stream(3, 2),
            source_clock: bounded(),
            sample_rate: 44_100,
            channels: 2,
        },
        MediaDerivation::Thumbnail {
            source: source(),
            source_stream: stream(4, 3),
            at: rational(1, 4),
            width: 320,
            height: 180,
        },
        MediaDerivation::Waveform {
            source: source(),
            source_stream: stream(5, 4),
            source_clock: bounded(),
            sample_rate: 22_050,
            width: 800,
            height: 120,
            color: "#12ab34".into(),
        },
        MediaDerivation::OpticalFlow {
            source: source(),
            source_stream: stream(6, 5),
            source_clock: bounded(),
            width: 960,
            height: 540,
            frame_rate: rational(60, 1),
            method: ProjectOpticalFlowMethod::MotionCompensated,
        },
        MediaDerivation::SourceSegment {
            source: source(),
            video_stream: stream(7, 6),
            start: rational(1, 4),
            duration: rational(1, 2),
            width: 1280,
            height: 720,
            frame_rate: rational(24, 1),
            audio: Some(ProjectSegmentAudio {
                source_stream: stream(8, 7),
                sample_rate: 48_000,
                channels: 6,
            }),
            crf: 19,
        },
    ]
}

fn source() -> InputId {
    InputId::from("source")
}

fn bounded() -> ProjectSourceClock {
    ProjectSourceClock::Bounded {
        start: rational(1, 4),
        duration: rational(1, 2),
    }
}

fn stream(global_index: u32, type_index: u32) -> ProjectStreamSelection {
    ProjectStreamSelection {
        global_index,
        type_index,
    }
}

fn rational(numerator: i64, denominator: u64) -> ProjectRational {
    ProjectRational::new(numerator, denominator)
}
