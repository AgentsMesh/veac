use veac_ir::RationalTime;

pub(super) fn stream_duration(
    duration: Option<RationalTime>,
    start_time: Option<RationalTime>,
    container: Option<RationalTime>,
) -> Option<RationalTime> {
    duration.or_else(|| {
        start_time
            .map(|time| time.value == 0)
            .unwrap_or(true)
            .then_some(container)
            .flatten()
    })
}
