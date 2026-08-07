use super::super::use_macro::define_control_uses;

define_control_uses! {
    PARAMETER_CURVE_KIND => "curve" @ ParameterCurveMember : KindDiscriminator;
    PARAMETER_KEY_MEMBER => "key" @ ParameterCurveMember : DeclarationIntroducer;
    PARAMETER_KEY_AT_FIELD => "at" @ ParameterKeyMember : FieldIntroducer;
    PARAMETER_KEY_VALUE_FIELD => "value" @ ParameterKeyMember : FieldIntroducer;
    PARAMETER_KEY_INTERPOLATION_FIELD => "interpolation" @ ParameterKeyMember : FieldIntroducer;
    EFFECT_PARAMETER_CURVE_KIND => "curve" @ EffectParameterCurveMember : KindDiscriminator;
    EFFECT_PARAMETER_KEY_MEMBER => "key" @ EffectParameterCurveMember : DeclarationIntroducer;
    EFFECT_PARAMETER_KEY_AT_FIELD => "at" @ EffectParameterKeyMember : FieldIntroducer;
    EFFECT_PARAMETER_KEY_VALUE_FIELD => "value" @ EffectParameterKeyMember : FieldIntroducer;
    EFFECT_PARAMETER_KEY_INTERPOLATION_FIELD => "interpolation" @ EffectParameterKeyMember : FieldIntroducer;
    CUBIC_X1_FIELD => "x1" @ CubicBezierInterpolationMember : FieldIntroducer;
    CUBIC_Y1_FIELD => "y1" @ CubicBezierInterpolationMember : FieldIntroducer;
    CUBIC_X2_FIELD => "x2" @ CubicBezierInterpolationMember : FieldIntroducer;
    CUBIC_Y2_FIELD => "y2" @ CubicBezierInterpolationMember : FieldIntroducer;
    SPRING_FREQUENCY_FIELD => "frequency" @ SpringInterpolationMember : FieldIntroducer;
    SPRING_DECAY_FIELD => "decay" @ SpringInterpolationMember : FieldIntroducer;
    SPRING_INITIAL_VELOCITY_FIELD => "initial-velocity" @ SpringInterpolationMember : FieldIntroducer;
}
