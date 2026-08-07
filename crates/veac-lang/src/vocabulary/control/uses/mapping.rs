use super::super::use_macro::define_control_uses;

define_control_uses! {
    LINEAR_FROM_FIELD => "from" @ LinearMappingMember : FieldIntroducer;
    LINEAR_TO_FIELD => "to" @ LinearMappingMember : FieldIntroducer;
    LINEAR_OUTSIDE_FIELD => "outside" @ LinearMappingMember : FieldIntroducer;
    CURVE_KEY_MEMBER => "key" @ CurveMappingMember : DeclarationIntroducer;
    CURVE_OUTSIDE_FIELD => "outside" @ CurveMappingMember : FieldIntroducer;
    FREEZE_SOURCE_FIELD => "source" @ FreezeMappingMember : FieldIntroducer;
    KEY_AT_FIELD => "at" @ MappingKeyMember : FieldIntroducer;
    KEY_SOURCE_FIELD => "source" @ MappingKeyMember : FieldIntroducer;
    KEY_INTERPOLATION_FIELD => "interpolation" @ MappingKeyMember : FieldIntroducer;
}
