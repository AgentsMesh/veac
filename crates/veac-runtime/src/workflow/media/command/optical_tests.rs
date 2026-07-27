use std::path::Path;

use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};

use super::*;

#[test]
fn bounded_optical_clock_is_applied_inside_the_exact_filter_timeline() {
    let values = args(SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(125), time(250)).unwrap(),
    });
    assert!(!values.iter().any(|value| value == "-ss" || value == "-t"));
    assert_pair(&values, "-map", "0:6");
    let filter = option(&values, "-vf");
    assert!(filter.starts_with("trim=start=1.25:duration=2.5,setpts=PTS-STARTPTS,"));
    assert!(filter.contains("scale=320:180"));
    assert!(filter.contains("tpad=stop_mode=clone:stop=2"));
    assert!(filter.contains("fps=24/1:mi_mode=mci:mc_mode=obmc"));
    assert!(filter.ends_with("trim=duration=2.5,setpts=PTS-STARTPTS"));
}

#[test]
fn identity_optical_clock_keeps_physical_output_at_zero() {
    let values = args(SourceClockSpec::Identity {
        duration: time(100),
    });
    let filter = option(&values, "-vf");
    assert!(filter.starts_with("trim=start=0:duration=1,setpts=PTS-STARTPTS,"));
    assert!(filter.ends_with("trim=duration=1,setpts=PTS-STARTPTS"));
}

fn args(source_clock: SourceClockSpec) -> Vec<String> {
    let spec = MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
        source_stream: StreamSelection {
            global_index: 6,
            type_index: 1,
        },
        source_clock,
        width: 320,
        height: 180,
        frame_rate: Rational::new(24, 1).unwrap(),
        method: OpticalFlowMethod::BlockMatching,
    });
    arguments(
        Path::new("input.mp4"),
        Path::new("output.mp4"),
        &spec,
        MediaArtifactLimits::default(),
    )
    .unwrap()
    .into_iter()
    .map(|value| value.into_string().unwrap())
    .collect()
}

fn option<'a>(values: &'a [String], name: &str) -> &'a str {
    values
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
        .unwrap()
}

fn assert_pair(values: &[String], name: &str, value: &str) {
    assert!(values
        .windows(2)
        .any(|pair| pair[0] == name && pair[1] == value));
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}
