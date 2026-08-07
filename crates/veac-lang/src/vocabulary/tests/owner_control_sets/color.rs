use crate::vocabulary::{control_uses::color as c, CanonicalRole as R, GrammarPosition as P};

use super::{assert_complete, assert_group};

#[test]
fn color_controls_have_exact_owners_and_roles() {
    let groups = [
        (
            &[
                c::MODIFIER_INPUT_SPACE_FIELD,
                c::MODIFIER_WORKING_SPACE_FIELD,
                c::MODIFIER_OUTPUT_SPACE_FIELD,
            ][..],
            P::ColorModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::BASIC_STAGE_MEMBER,
                c::MATRIX_STAGE_MEMBER,
                c::HSL_STAGE_MEMBER,
                c::CURVES_STAGE_MEMBER,
                c::WHEELS_STAGE_MEMBER,
                c::LUT_STAGE_MEMBER,
            ],
            P::ColorModifierMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::BASIC_EXPOSURE_FIELD,
                c::BASIC_HIGHLIGHTS_FIELD,
                c::BASIC_SHADOWS_FIELD,
                c::BASIC_TEMPERATURE_FIELD,
                c::BASIC_TINT_FIELD,
                c::BASIC_FADE_FIELD,
            ],
            P::BasicColorMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::MATRIX_RED_FIELD,
                c::MATRIX_GREEN_FIELD,
                c::MATRIX_BLUE_FIELD,
                c::MATRIX_OFFSET_FIELD,
            ],
            P::RgbMatrixMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::HSL_RANGE_FIELD,
                c::HSL_HUE_FIELD,
                c::HSL_SATURATION_FIELD,
                c::HSL_LIGHTNESS_FIELD,
            ],
            P::HslColorMember,
            R::FieldIntroducer,
        ),
        (
            &[c::CURVES_INTERPOLATION_FIELD],
            P::ColorCurvesMember,
            R::FieldIntroducer,
        ),
        (
            &[c::CURVES_CURVE_MEMBER],
            P::ColorCurvesMember,
            R::DeclarationIntroducer,
        ),
        (
            &[c::CURVE_POINT_MEMBER],
            P::ColorCurveMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::WHEELS_LIFT_FIELD,
                c::WHEELS_GAMMA_FIELD,
                c::WHEELS_GAIN_FIELD,
            ],
            P::ColorWheelsMember,
            R::FieldIntroducer,
        ),
        (
            &[c::LUT_INTERPOLATION_FIELD],
            P::LutColorMember,
            R::FieldIntroducer,
        ),
        (
            &[c::LUT_RESOURCE_REFERENCE_KIND],
            P::LutResourceReferenceKind,
            R::ReferenceKind,
        ),
    ];
    let expected = groups
        .iter()
        .flat_map(|(values, _, _)| *values)
        .copied()
        .collect::<Vec<_>>();
    for (controls, position, role) in groups {
        assert_group(controls, position, role);
    }
    assert_complete(&expected, c::ALL);
}

#[test]
fn repeated_color_spellings_keep_distinct_owners() {
    assert_eq!(
        c::CURVES_INTERPOLATION_FIELD.word(),
        c::LUT_INTERPOLATION_FIELD.word()
    );
    assert_ne!(c::CURVES_INTERPOLATION_FIELD, c::LUT_INTERPOLATION_FIELD);
}
