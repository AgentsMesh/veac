use super::*;

#[test]
fn selected_stream_range_is_normalized_to_project_ticks() {
    let clock = resolve(
        &id(),
        1_000,
        Some(time(100)),
        Some(time(5)),
        Some(time(10)),
        "video",
    )
    .unwrap();
    assert_eq!(
        clock,
        SourceClockSpec::Bounded {
            logical_range: TimeRange::new(ticks(5_000), ticks(10_000)).unwrap(),
        }
    );
}

#[test]
fn zero_origin_stream_and_container_fallbacks_are_exact_identities() {
    assert_eq!(
        resolve(&id(), 1_000, None, Some(time(0)), Some(time(10)), "video",).unwrap(),
        SourceClockSpec::Identity {
            duration: ticks(10_000),
        }
    );
    assert_eq!(
        resolve(&id(), 1_000, Some(time(20)), None, None, "audio").unwrap(),
        SourceClockSpec::Identity {
            duration: ticks(20_000),
        }
    );
}

#[test]
fn exact_mixed_scales_align_but_unavailable_ranges_fail_closed() {
    let aligned = resolve(
        &id(),
        1_000,
        None,
        Some(time(5)),
        Some(RationalTime::new(10, 2).unwrap()),
        "audio",
    )
    .unwrap();
    assert_eq!(
        aligned,
        SourceClockSpec::Bounded {
            logical_range: TimeRange::new(ticks(5_000), ticks(5_000)).unwrap(),
        }
    );

    for result in [
        resolve(&id(), 1_000, Some(time(20)), Some(time(5)), None, "video"),
        resolve(
            &id(),
            2,
            None,
            Some(RationalTime::new(veac_ir::MAX_SAFE_INTEGER as i64, 1).unwrap()),
            Some(RationalTime::new(1, 2).unwrap()),
            "video",
        ),
        resolve(
            &id(),
            1_000,
            None,
            Some(RationalTime::new(1, 3).unwrap()),
            Some(time(1)),
            "video",
        ),
    ] {
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("PROXY_DURATION_UNAVAILABLE"));
    }
}

fn id() -> PlanInputId {
    PlanInputId::new("pin_test").unwrap()
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 1).unwrap()
}

fn ticks(value: i64) -> RationalTime {
    RationalTime::new(value, 1_000).unwrap()
}
