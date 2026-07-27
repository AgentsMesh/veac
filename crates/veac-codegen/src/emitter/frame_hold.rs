use veac_plan::canonical::{Rational, RationalTime};

use super::time;

pub(super) fn filter(
    source_time: RationalTime,
    record_duration: RationalTime,
    image: bool,
    frame_rate: Rational,
) -> String {
    let fps = format!("{}/{}", frame_rate.numerator, frame_rate.denominator);
    let selector = if image {
        format!("loop=loop=-1:size=1:start=0,fps=fps={fps}")
    } else {
        format!(
            "fps=fps={fps}:start_time={}:round=down:eof_action=pass",
            time::seconds(source_time)
        )
    };
    format!(
        "{selector},trim=end_frame=1,setpts=PTS-STARTPTS,tpad=stop_mode=clone:stop_duration={},trim=duration={}",
        time::seconds(record_duration),
        time::seconds(record_duration),
    )
}
