use crate::{time::*, *};

fn otio(value: f64, rate: f64) -> OtioRationalTime {
    OtioRationalTime {
        schema: OTIO_RATIONAL_TIME_SCHEMA.to_owned(),
        value,
        rate,
    }
}

#[test]
fn invalid_canonical_and_otio_time_domains_are_rejected() {
    assert!(export_time(veac_ir::RationalTime {
        value: 0,
        timescale: 0,
    })
    .is_err());
    assert!(export_time(veac_ir::RationalTime {
        value: i64::MAX,
        timescale: 600,
    })
    .is_err());
    for value in [
        otio(f64::NAN, 24.0),
        otio(1.0, f64::INFINITY),
        otio(1.0, -24.0),
        otio(veac_ir::MAX_SAFE_INTEGER as f64 * 2.0, 24.0),
        otio(1.0, veac_ir::MAX_SAFE_INTEGER as f64 * 2.0),
    ] {
        assert!(validate_time(&value).is_err());
        assert!(import_time(value, 600).is_err());
    }
    assert!(import_time(otio(1.0, 24.0), 0).is_err());
    assert!(import_time(otio(1e-40, 24.0), 600).is_err());
    let maximum = veac_ir::MAX_SAFE_INTEGER as f64;
    assert!(import_time(otio(maximum, 1e-30), u32::MAX).is_err());
    assert!(import_time(otio(1e-30, maximum), 1)
        .unwrap_err()
        .to_string()
        .contains("arithmetic overflowed"));
    assert!(import_time(otio(maximum, 1.0), 2).is_err());
    assert!(import_time(otio(maximum, 1.0), u32::MAX).is_err());
}

#[test]
fn range_schema_duration_and_exact_conversion_are_enforced() {
    let mut range = OtioTimeRange {
        schema: OTIO_TIME_RANGE_SCHEMA.to_owned(),
        start_time: otio(-1.0, 30.0),
        duration: otio(1.0, 30.0),
    };
    assert!(validate_range(&range).is_ok());
    assert_eq!(import_range(range.clone(), 600).unwrap().start.value, -20);
    range.duration.value = 0.0;
    assert!(validate_range(&range).is_err());
    assert!(import_range(range.clone(), 600).is_err());
    range.duration.value = 1.0;
    range.schema = "TimeRange.9".to_owned();
    assert!(import_range(range, 600).is_err());
    assert!(import_time(otio(0.1, 24.0), 600).is_err());
}
