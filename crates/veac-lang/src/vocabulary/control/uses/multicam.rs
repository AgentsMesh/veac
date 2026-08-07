use super::super::use_macro::define_control_uses;

define_control_uses! {
    SYNC_FIELD => "sync" @ MulticamMember : FieldIntroducer;
    SYNC_REFERENCE_FIELD => "reference" @ MulticamSyncMember : FieldIntroducer;
    SYNC_ANGLE_REFERENCE_KIND => "angle" @ MulticamSyncReferenceKind : ReferenceKind;
    ANGLE_MEMBER => "angle" @ MulticamMember : DeclarationIntroducer;
    ANGLE_SOURCE_FIELD => "source" @ MulticamAngleMember : FieldIntroducer;
    ANGLE_SOURCE_OFFSET_FIELD => "source-offset" @ MulticamAngleMember : FieldIntroducer;
    ANGLE_RESOURCE_REFERENCE_KIND => "resource" @ MulticamAngleSourceReferenceKind : ReferenceKind;
    SWITCH_MEMBER => "switch" @ MulticamSourceMember : DeclarationIntroducer;
    SWITCH_ANGLE_REFERENCE_KIND => "angle" @ MulticamSwitchReferenceKind : ReferenceKind;
    SWITCH_AT_FIELD => "at" @ MulticamSwitchMember : FieldIntroducer;
    SWITCH_DURATION_FIELD => "duration" @ MulticamSwitchMember : FieldIntroducer;
}
