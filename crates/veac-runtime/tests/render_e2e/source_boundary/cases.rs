use super::ExpectedColor;
use crate::support::SourceOutOfRangePolicy;

pub(super) struct BoundaryCase {
    pub name: &'static str,
    pub start: i64,
    pub policy: SourceOutOfRangePolicy,
    pub pixel_time: f64,
    pub color: ExpectedColor,
    pub silent_at: f64,
    pub signal_at: f64,
    pub video_filter: &'static str,
    pub audio_filter: &'static str,
}

impl BoundaryCase {
    pub(super) fn first() -> Self {
        Self {
            name: "hold-first",
            start: -500,
            policy: SourceOutOfRangePolicy::HoldFirst,
            pixel_time: 0.25,
            color: ExpectedColor::Red,
            silent_at: 0.1,
            signal_at: 0.8,
            video_filter: "start_mode=clone",
            audio_filter: "adelay=24000S:all=1",
        }
    }

    pub(super) fn last() -> Self {
        Self {
            name: "hold-last",
            start: 3_500,
            policy: SourceOutOfRangePolicy::HoldLast,
            pixel_time: 1.25,
            color: ExpectedColor::Yellow,
            silent_at: 1.1,
            signal_at: 0.1,
            video_filter: "stop_mode=clone",
            audio_filter: "apad=pad_len=48000",
        }
    }
}
