use super::super::CoreTemporalInputIdentity;
use veac_ir::{TemporalClock, TemporalInputSource};

pub(super) fn source(value: &CoreTemporalInputIdentity) -> TemporalInputSource {
    use CoreTemporalInputIdentity::*;
    let clock = match value {
        SequenceTime { .. } => Some(TemporalClock::SequenceTime),
        ClipTime { .. } => Some(TemporalClock::ClipTime),
        SourceTime { .. } => Some(TemporalClock::SourceTime),
        Frame { .. } => Some(TemporalClock::Frame),
        Progress { .. } => Some(TemporalClock::Progress),
        Parameter { parameter_id, .. } => {
            return TemporalInputSource::Parameter {
                parameter_id: parameter_id.clone(),
            }
        }
    };
    TemporalInputSource::Clock {
        clock: clock.expect("clock identity maps to a clock"),
    }
}
