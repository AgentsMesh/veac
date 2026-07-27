use crate::{time::*, *};

#[test]
fn exact_time_conversion_round_trips_and_rejects_inexact_values() {
    let canonical = veac_ir::RationalTime::new(1234, 600).unwrap();
    assert_eq!(
        import_time(export_time(canonical).unwrap(), 600).unwrap(),
        canonical
    );
    let range =
        veac_ir::TimeRange::new(canonical, veac_ir::RationalTime::new(60, 600).unwrap()).unwrap();
    assert_eq!(
        import_range(export_range(range).unwrap(), 600).unwrap(),
        range
    );
    assert!(import_time(
        OtioRationalTime {
            schema: OTIO_RATIONAL_TIME_SCHEMA.to_owned(),
            value: 1.0,
            rate: 24.0,
        },
        1000,
    )
    .is_err());
    assert!(import_time(
        OtioRationalTime {
            schema: "RationalTime.9".to_owned(),
            value: f64::INFINITY,
            rate: 0.0,
        },
        0,
    )
    .is_err());
}
