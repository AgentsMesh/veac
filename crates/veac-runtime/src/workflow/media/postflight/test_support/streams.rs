use veac_artifact::SourceClockSpec;
use veac_ir::*;

pub(in crate::workflow::media::postflight) fn video(
    codec: &str,
    width: u32,
    height: u32,
    frame_rate: Option<Rational>,
    duration: Option<RationalTime>,
    origin: bool,
) -> ProbedStream {
    ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: codec.into(),
        time_base: Some(Rational::new(1, 1_000).unwrap()),
        start_time: origin.then(|| time(0)),
        duration,
        disposition: disposition(),
        video: Some(VideoStreamInfo {
            width,
            height,
            frame_rate,
            cadence: if frame_rate.is_some() {
                VideoCadence::Constant
            } else {
                VideoCadence::Unknown
            },
            pixel_format: "yuv420p".into(),
            profile: None,
            level: None,
            sample_aspect_ratio: rate(1),
            rotation_degrees: 0,
        }),
        audio: None,
    }
}

pub(in crate::workflow::media::postflight) fn audio(
    codec: &str,
    sample_rate: u32,
    channels: u8,
    duration: Option<RationalTime>,
    origin: bool,
) -> ProbedStream {
    ProbedStream {
        global_index: 1,
        type_index: 0,
        media_type: ProbedStreamType::Audio,
        codec: codec.into(),
        time_base: Some(Rational::new(1, sample_rate).unwrap()),
        start_time: origin.then(|| time(0)),
        duration,
        disposition: disposition(),
        video: None,
        audio: Some(AudioStreamInfo {
            sample_rate,
            channels,
            channel_layout: "stereo".into(),
        }),
    }
}

pub(in crate::workflow::media::postflight) fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

pub(in crate::workflow::media::postflight) fn identity(duration: i64) -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: time(duration),
    }
}

pub(in crate::workflow::media::postflight) fn selection(global_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index: 0,
    }
}

pub(in crate::workflow::media::postflight) fn rate(value: i64) -> Rational {
    Rational::new(value, 1).unwrap()
}

fn disposition() -> StreamDisposition {
    StreamDisposition {
        default: true,
        attached_picture: false,
        timed_thumbnail: false,
    }
}
