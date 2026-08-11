use tempfile::tempdir;

#[path = "directional_blur_identity/fixture.rs"]
mod fixture;
#[path = "directional_blur_identity/yuv_transform.rs"]
mod yuv_transform;
use fixture::*;

#[test]
fn zero_radius_and_inactive_windows_are_exact_in_sixteen_bit_alpha_delivery() {
    let temp = tempdir().unwrap();
    let source = alpha_fixture(temp.path());
    let baseline = render_frames(temp.path(), &source, "baseline", None);
    let zero = render_frames(
        temp.path(),
        &source,
        "zero",
        Some(effect("zero", constant(0.0), None)),
    );
    let window = render_frames(
        temp.path(),
        &source,
        "window",
        Some(effect("window", constant(12.0), Some((300, 400)))),
    );
    let dynamic = render_frames(
        temp.path(),
        &source,
        "dynamic",
        Some(effect("dynamic", zero_crossing(), None)),
    );
    assert_high_precision_alpha(&baseline[0]);

    for index in 0..FRAMES {
        assert_eq!(baseline[index], zero[index], "static zero frame {index}");
    }
    for index in [0, 1, 2, 7, 8, 9] {
        assert_eq!(baseline[index], window[index], "window frame {index}");
        assert_eq!(
            baseline[index], dynamic[index],
            "dynamic zero frame {index}"
        );
    }
    for index in 3..7 {
        assert_ne!(
            baseline[index], window[index],
            "active window frame {index}"
        );
        assert_ne!(
            baseline[index], dynamic[index],
            "dynamic blur frame {index}"
        );
    }
}
