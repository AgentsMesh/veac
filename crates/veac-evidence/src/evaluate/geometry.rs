use crate::{
    bounds as metric_bounds, mask, AssertionKind, AssertionResult, BoundsSpec, LayerOrderSpec,
    ObservationSet,
};

use super::{error, region, result, status};

pub(super) fn bounds(
    value: &BoundsSpec,
    suite: &crate::EvidenceSuiteV1,
    set: &ObservationSet,
) -> AssertionResult {
    let Some(frame) = set.frames.get(&value.sample_id) else {
        return error(&value.id, AssertionKind::Bounds, "sample frame is missing");
    };
    let reference = match &value.mask {
        crate::MaskSpec::Difference {
            reference_sample_id,
            ..
        } => set.frames.get(reference_sample_id),
        _ => None,
    };
    let mask = match mask(
        frame,
        reference,
        region(suite, &value.region_id),
        &value.mask,
    ) {
        Ok(value) => value,
        Err(problem) => return error(&value.id, AssertionKind::Bounds, problem.to_string()),
    };
    let measured = metric_bounds(&mask, frame.width, frame.height);
    let passed = bounds_pass(measured, &value.expectation);
    let mut output = result(
        &value.id,
        AssertionKind::Bounds,
        status(passed),
        "bounds evaluated",
    );
    if let Some(value) = measured {
        output.metrics.extend([
            ("x".into(), f64::from(value.x)),
            ("y".into(), f64::from(value.y)),
            ("width".into(), f64::from(value.width)),
            ("height".into(), f64::from(value.height)),
            ("minimum_margin".into(), f64::from(minimum_margin(value))),
        ]);
    }
    output
}

pub(super) fn layer(value: &LayerOrderSpec, set: &ObservationSet) -> AssertionResult {
    let Some(observed) = set.layer_orders.get(&value.id) else {
        return error(
            &value.id,
            AssertionKind::LayerOrder,
            "layer order observation is missing",
        );
    };
    let passed = observed.coactive
        && observed.upper_entity == value.upper_entity
        && observed.lower_entity == value.lower_entity
        && observed.upper_order > observed.lower_order;
    let mut output = result(
        &value.id,
        AssertionKind::LayerOrder,
        status(passed),
        "layer order evaluated",
    );
    output
        .metrics
        .insert("upper_order".into(), observed.upper_order as f64);
    output
        .metrics
        .insert("lower_order".into(), observed.lower_order as f64);
    output
}

fn bounds_pass(value: Option<crate::Bounds>, expected: &crate::BoundsExpectation) -> bool {
    match value {
        None => !expected.non_empty,
        Some(_) if !expected.non_empty => false,
        Some(value) => {
            expected
                .minimum_margin_pixels
                .is_none_or(|minimum| minimum_margin(value) >= minimum)
                && expected
                    .maximum_width_pixels
                    .is_none_or(|maximum| value.width <= maximum)
                && expected
                    .maximum_height_pixels
                    .is_none_or(|maximum| value.height <= maximum)
        }
    }
}

fn minimum_margin(value: crate::Bounds) -> u32 {
    value
        .margin_left
        .min(value.margin_top)
        .min(value.margin_right)
        .min(value.margin_bottom)
}
