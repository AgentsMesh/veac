use super::{styled_item, transform};
use crate::support;

#[test]
fn invalid_transform_channel_bounds_fail_closed() {
    for (position, scale, rotation, pivot) in [
        (
            "point(9007199254740992px, 0px)",
            "vector(1.0, 1.0)",
            "0deg",
            "vector(0.5, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(1.0, 1.0)",
            "-9007199254740992deg",
            "vector(0.5, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(0.0, 1.0)",
            "0deg",
            "vector(0.5, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(-1.0, 1.0)",
            "0deg",
            "vector(0.5, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(16.01, 1.0)",
            "0deg",
            "vector(0.5, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(1.0 / 9007199254740992.0, 1.0)",
            "0deg",
            "vector(0.5, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(1.0, 1.0)",
            "0deg",
            "vector(-0.01, 0.5)",
        ),
        (
            "point(0px, 0px)",
            "vector(1.0, 1.0)",
            "0deg",
            "vector(0.5, 1.01)",
        ),
    ] {
        let value = transform(position, scale, rotation, pivot, "flip_none()");
        let item = styled_item("card", &value);
        let error = support::error(&support::visual_project(&[&item]));
        assert!(
            matches!(
                error.code,
                "PROGRAM_EXECUTABLE_LOWER" | "PROGRAM_EXECUTABLE_IR"
            ),
            "{value}: {error:?}"
        );
        assert!(
            error.message.contains("EXECUTABLE_LOWER_ANIMATION")
                || error.message.contains("EXECUTABLE_LOWER_TRANSFORM")
                || error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"),
            "{value}: {}",
            error.message
        );
    }
}

#[test]
fn exact_transform_arithmetic_overflow_fails_before_publication() {
    for (position, scale) in [
        (
            "point(170141183460469231731687303715884105727px + 1px, 0px)",
            "vector(1.0, 1.0)",
        ),
        (
            "point(0px, 0px)",
            "vector(170141183460469231731687303715884105727.0 * 2.0, 1.0)",
        ),
    ] {
        let value = transform(position, scale, "0deg", "vector(0.5, 0.5)", "flip_none()");
        let item = styled_item("card", &value);
        let result = veac_lang::program::build_source(&support::visual_project(&[&item]));
        let error = result.expect_err("overflow must fail before canonical publication");
        assert!(
            error.as_slice()[0].message.contains("EXPRESSION_OVERFLOW"),
            "{value}: {error}"
        );
    }
}
