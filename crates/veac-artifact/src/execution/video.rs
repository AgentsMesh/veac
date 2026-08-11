use veac_ir::{Rational, VideoCadence, VideoStreamInfo};

use crate::ProxyVideoSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundVideoFacts {
    width: u32,
    height: u32,
    frame_rate: Option<Rational>,
    cadence: VideoCadence,
}

impl BoundVideoFacts {
    pub(super) fn original(info: &VideoStreamInfo) -> Self {
        Self {
            width: info.width,
            height: info.height,
            frame_rate: info.frame_rate,
            cadence: info.cadence,
        }
    }

    pub(super) fn proxy(spec: &ProxyVideoSpec) -> Self {
        Self {
            width: spec.width,
            height: spec.height,
            frame_rate: Some(spec.frame_rate),
            cadence: VideoCadence::Constant,
        }
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }

    pub fn frame_rate(self) -> Option<Rational> {
        self.frame_rate
    }

    pub fn cadence(self) -> VideoCadence {
        self.cadence
    }
}
