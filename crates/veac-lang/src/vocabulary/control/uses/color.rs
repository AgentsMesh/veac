use super::super::use_macro::define_control_uses;

define_control_uses! {
    MODIFIER_INPUT_SPACE_FIELD => "input-space" @ ColorModifierMember : FieldIntroducer;
    MODIFIER_WORKING_SPACE_FIELD => "working-space" @ ColorModifierMember : FieldIntroducer;
    MODIFIER_OUTPUT_SPACE_FIELD => "output-space" @ ColorModifierMember : FieldIntroducer;
    BASIC_STAGE_MEMBER => "basic" @ ColorModifierMember : DeclarationIntroducer;
    MATRIX_STAGE_MEMBER => "matrix" @ ColorModifierMember : DeclarationIntroducer;
    HSL_STAGE_MEMBER => "hsl" @ ColorModifierMember : DeclarationIntroducer;
    CURVES_STAGE_MEMBER => "curves" @ ColorModifierMember : DeclarationIntroducer;
    WHEELS_STAGE_MEMBER => "wheels" @ ColorModifierMember : DeclarationIntroducer;
    LUT_STAGE_MEMBER => "lut" @ ColorModifierMember : DeclarationIntroducer;
    BASIC_EXPOSURE_FIELD => "exposure" @ BasicColorMember : FieldIntroducer;
    BASIC_HIGHLIGHTS_FIELD => "highlights" @ BasicColorMember : FieldIntroducer;
    BASIC_SHADOWS_FIELD => "shadows" @ BasicColorMember : FieldIntroducer;
    BASIC_TEMPERATURE_FIELD => "temperature" @ BasicColorMember : FieldIntroducer;
    BASIC_TINT_FIELD => "tint" @ BasicColorMember : FieldIntroducer;
    BASIC_FADE_FIELD => "fade" @ BasicColorMember : FieldIntroducer;
    MATRIX_RED_FIELD => "red" @ RgbMatrixMember : FieldIntroducer;
    MATRIX_GREEN_FIELD => "green" @ RgbMatrixMember : FieldIntroducer;
    MATRIX_BLUE_FIELD => "blue" @ RgbMatrixMember : FieldIntroducer;
    MATRIX_OFFSET_FIELD => "offset" @ RgbMatrixMember : FieldIntroducer;
    HSL_RANGE_FIELD => "range" @ HslColorMember : FieldIntroducer;
    HSL_HUE_FIELD => "hue" @ HslColorMember : FieldIntroducer;
    HSL_SATURATION_FIELD => "saturation" @ HslColorMember : FieldIntroducer;
    HSL_LIGHTNESS_FIELD => "lightness" @ HslColorMember : FieldIntroducer;
    CURVES_INTERPOLATION_FIELD => "interpolation" @ ColorCurvesMember : FieldIntroducer;
    CURVES_CURVE_MEMBER => "curve" @ ColorCurvesMember : DeclarationIntroducer;
    CURVE_POINT_MEMBER => "point" @ ColorCurveMember : DeclarationIntroducer;
    WHEELS_LIFT_FIELD => "lift" @ ColorWheelsMember : FieldIntroducer;
    WHEELS_GAMMA_FIELD => "gamma" @ ColorWheelsMember : FieldIntroducer;
    WHEELS_GAIN_FIELD => "gain" @ ColorWheelsMember : FieldIntroducer;
    LUT_RESOURCE_REFERENCE_KIND => "resource" @ LutResourceReferenceKind : ReferenceKind;
    LUT_INTERPOLATION_FIELD => "interpolation" @ LutColorMember : FieldIntroducer;
}
