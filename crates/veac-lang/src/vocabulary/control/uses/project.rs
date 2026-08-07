use super::super::use_macro::define_control_uses;

define_control_uses! {
    PROJECT_DECLARATION => "project" @ ProjectDeclaration : DeclarationIntroducer;
    ENTRY_FIELD => "entry" @ ProjectMember : FieldIntroducer;
    ENTRY_SEQUENCE_KIND => "sequence" @ ProjectEntryReferenceKind : ReferenceKind;
    SETTINGS_FIELD => "settings" @ ProjectMember : FieldIntroducer;
    RESOURCE_MEMBER => "resource" @ ProjectMember : DeclarationIntroducer;
    MULTICAM_MEMBER => "multicam" @ ProjectMember : DeclarationIntroducer;
    SEQUENCE_MEMBER => "sequence" @ ProjectMember : DeclarationIntroducer;
    ANNOTATION_MEMBER => "annotation" @ ProjectMember : DeclarationIntroducer;
    DELIVERY_MEMBER => "delivery" @ ProjectMember : DeclarationIntroducer;
    SETTINGS_TIMEBASE_FIELD => "timebase" @ SettingsMember : FieldIntroducer;
    SETTINGS_CANVAS_FIELD => "canvas" @ SettingsMember : FieldIntroducer;
    SETTINGS_FRAME_RATE_FIELD => "frame-rate" @ SettingsMember : FieldIntroducer;
    SETTINGS_SAMPLE_RATE_FIELD => "sample-rate" @ SettingsMember : FieldIntroducer;
    SETTINGS_CANVAS_BY => "by" @ SettingsCanvasSeparator : InfixSeparator;
}
