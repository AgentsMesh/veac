use super::super::use_macro::define_control_uses;

define_control_uses! {
    SCOPE_FIELD => "scope" @ ApplyMember : FieldIntroducer;
    RECORD_FIELD => "record" @ ApplyMember : FieldIntroducer;
    PIPELINE_FIELD => "pipeline" @ ApplyMember : FieldIntroducer;
    MIX_FIELD => "mix" @ ApplyMember : FieldIntroducer;
    STAGE_MEMBER => "stage" @ ApplyPipelineMember : DeclarationIntroducer;
    BAND_FROM_FIELD => "from" @ ApplyCompositeBandMember : FieldIntroducer;
    BAND_THROUGH_FIELD => "through" @ ApplyCompositeBandMember : FieldIntroducer;
    BAND_LAYER_REFERENCE => "layer" @ ApplyCompositeBandReferenceKind : ReferenceKind;
    ITEMS_ITEM_REFERENCE => "item" @ ApplyItemsMember : ReferenceKind;
    ITEMS_GROUP_REFERENCE => "group" @ ApplyItemsMember : ReferenceKind;
    MIX_OPACITY_FIELD => "opacity" @ ApplyMixMember : FieldIntroducer;
    MIX_BLEND_FIELD => "blend" @ ApplyMixMember : FieldIntroducer;
    MIX_MASK_MEMBER => "mask" @ ApplyMixMember : DeclarationIntroducer;
}
