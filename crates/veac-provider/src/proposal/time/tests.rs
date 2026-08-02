use super::*;

#[test]
fn ranges_convert_only_when_exactly_representable() {
    let source = TimeRange::new(
        RationalTime::new(25, 100).unwrap(),
        RationalTime::new(50, 100).unwrap(),
    )
    .unwrap();
    assert_eq!(
        range_to_timebase(source, 600).unwrap(),
        TimeRange::new(
            RationalTime::new(150, 600).unwrap(),
            RationalTime::new(300, 600).unwrap(),
        )
        .unwrap()
    );
    assert_eq!(
        range_to_timebase(source, 30).unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
    assert_eq!(
        range_to_timebase(source, 0).unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );

    let negative_duration = TimeRange {
        start: RationalTime::new(0, 1).unwrap(),
        duration: RationalTime::new(-1, 1).unwrap(),
    };
    let error = range_to_timebase(negative_duration, 100).unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert!(std::error::Error::source(&error).is_some());

    let overflow = TimeRange {
        start: RationalTime {
            value: i64::MAX,
            timescale: 1,
        },
        duration: RationalTime::new(1, 1).unwrap(),
    };
    assert!(range_to_timebase(overflow, u32::MAX)
        .unwrap_err()
        .to_string()
        .contains("overflows"));

    let unsafe_integer = TimeRange {
        start: RationalTime::new(veac_ir::MAX_SAFE_INTEGER as i64, 1).unwrap(),
        duration: RationalTime::new(1, 1).unwrap(),
    };
    let error = range_to_timebase(unsafe_integer, 2).unwrap_err();
    assert!(error
        .to_string()
        .contains("invalid in the project timebase"));
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn clip_mapping_rejects_a_result_outside_the_exact_integer_domain() {
    let limit = veac_ir::MAX_SAFE_INTEGER as i64;
    let error = clip_time(
        RationalTime::new(limit, 1).unwrap(),
        super::super::ClipTimeBinding {
            provider_origin: RationalTime::new(-limit, 1).unwrap(),
            clip_local_origin: RationalTime::new(limit, 1).unwrap(),
        },
        1,
    )
    .unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert!(error.to_string().contains("mapping is invalid"));
    assert!(std::error::Error::source(&error).is_some());
}
