use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};
use veac_runtime::asset::selected_video;
use veac_runtime::workflow::MediaWorkflow;

use super::support::*;

#[test]
fn optical_flow_preserves_identity_and_bounded_clocks_exactly() {
    let temp = tempfile::tempdir().unwrap();
    let input = media_fixture(temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let workflow = MediaWorkflow::new("ffmpeg");
    let cases = [
        (
            SourceClockSpec::Identity {
                duration: time(100),
            },
            time(100),
            24,
        ),
        (
            SourceClockSpec::Bounded {
                logical_range: TimeRange::new(time(25), time(50)).unwrap(),
            },
            time(50),
            12,
        ),
    ];
    let mut first_frames = Vec::new();
    for (clock, duration, frames) in cases {
        let request = request(&input, optical(clock));
        let descriptor = request.descriptor().unwrap();
        let created = workflow.derive(&store, &input, &request).unwrap();
        let probe = probe_artifact(&store, &descriptor, &created.record);
        let stream = selected_video(&probe).unwrap();
        assert_eq!(stream.start_time.unwrap().value, 0);
        assert_eq!(stream.video.as_ref().unwrap().frame_rate, Some(rate(24)));
        assert_same_time(
            stream.duration.or(probe.container_duration).unwrap(),
            duration,
        );
        assert_eq!(
            video_frame_count(&store, &descriptor, &created.record),
            frames
        );
        first_frames.push(first_video_frame_identity(
            &store,
            &descriptor,
            &created.record,
        ));

        let cached = workflow.derive(&store, &input, &request).unwrap();
        assert!(cached.cache_hit);
        assert_eq!(cached.record, created.record);
    }
    assert_ne!(first_frames[0], first_frames[1]);
}

fn optical(source_clock: SourceClockSpec) -> MediaArtifactSpec {
    MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
        source_stream: StreamSelection {
            global_index: 1,
            type_index: 1,
        },
        source_clock,
        width: 120,
        height: 68,
        frame_rate: rate(24),
        method: OpticalFlowMethod::BlockMatching,
    })
}

fn assert_same_time(actual: RationalTime, expected: RationalTime) {
    assert_eq!(
        i128::from(actual.value) * i128::from(expected.timescale),
        i128::from(expected.value) * i128::from(actual.timescale)
    );
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn rate(value: i64) -> Rational {
    Rational::new(value, 1).unwrap()
}
