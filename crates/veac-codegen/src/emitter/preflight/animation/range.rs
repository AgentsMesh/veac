use veac_plan::canonical::{RationalTime, TimeRange};

pub(super) fn point(value: RationalTime, domain: RationalTime, timebase: u32) -> bool {
    value.is_valid()
        && domain.is_valid()
        && value.value >= 0
        && domain.value > 0
        && value.timescale == timebase
        && domain.timescale == timebase
        && value.value <= domain.value
}

pub(super) fn valid(range: TimeRange, domain: RationalTime, timebase: u32) -> bool {
    range.start.is_valid()
        && range.duration.is_valid()
        && range.start.value >= 0
        && range.duration.value > 0
        && range.start.timescale == timebase
        && range.duration.timescale == timebase
        && domain.is_valid()
        && domain.timescale == timebase
        && range
            .end()
            .is_ok_and(|end| end.value <= domain.value && end.timescale == timebase)
}
