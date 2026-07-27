use super::TextError;

const MAX_DIALOGUE_EVENTS: usize = 4_096;
const MAX_ANIMATION_UNIT_SAMPLES: usize = 262_144;

pub(super) fn samples(value: Option<usize>) -> Result<usize, TextError> {
    let value = value.ok_or_else(|| event_limit(usize::MAX))?;
    events(value)?;
    Ok(value)
}

pub(super) fn frame_rate(samples: usize, fps: f64) -> Result<(), TextError> {
    if samples > 1 && fps > 100.0 {
        Err(TextError::new(
            "TEXT_ASS_TIMEBASE_LIMIT",
            "animated ASS text requires a sequence frame rate at or below 100 fps",
        ))
    } else {
        Ok(())
    }
}

pub(super) fn unit_samples(frames: usize, units: usize) -> Result<(), TextError> {
    if frames
        .checked_mul(units)
        .is_none_or(|value| value > MAX_ANIMATION_UNIT_SAMPLES)
    {
        Err(TextError::new(
            "TEXT_ANIMATION_SAMPLE_LIMIT",
            "animated text exceeds the bounded frame-by-unit sampling budget",
        ))
    } else {
        Ok(())
    }
}

pub(super) fn events(value: usize) -> Result<(), TextError> {
    if value > MAX_DIALOGUE_EVENTS {
        Err(event_limit(value))
    } else {
        Ok(())
    }
}

pub(super) fn script(bytes: usize) -> Result<(), TextError> {
    if bytes > crate::emitter::MAX_ASS_PAYLOAD_BYTES {
        Err(TextError::new(
            "TEXT_ASS_PAYLOAD_LIMIT",
            format!(
                "ASS payload is {bytes} bytes; limit is {}",
                crate::emitter::MAX_ASS_PAYLOAD_BYTES
            ),
        ))
    } else {
        Ok(())
    }
}

fn event_limit(events: usize) -> TextError {
    TextError::new(
        "TEXT_ASS_EVENT_LIMIT",
        format!("animated text requires {events} ASS events; limit is {MAX_DIALOGUE_EVENTS}"),
    )
}
