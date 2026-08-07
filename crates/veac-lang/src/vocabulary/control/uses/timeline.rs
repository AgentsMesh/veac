use super::super::use_macro::define_control_uses;

define_control_uses! {
    LAYER_MEMBER => "layer" @ SequenceMember : DeclarationIntroducer;
    RELATION_MEMBER => "relation" @ SequenceMember : DeclarationIntroducer;
    APPLY_MEMBER => "apply" @ SequenceMember : DeclarationIntroducer;
    LAYER_PLACEMENT_FIELD => "placement" @ LayerMember : FieldIntroducer;
    LAYER_STATE_FIELD => "state" @ LayerMember : FieldIntroducer;
    LAYER_ORDER_FIELD => "order" @ LayerMember : FieldIntroducer;
    LAYER_ROUTE_FIELD => "route" @ LayerMember : FieldIntroducer;
    LAYER_ROUTE_BUS_KIND => "bus" @ LayerRouteReferenceKind : ReferenceKind;
    ITEM_MEMBER => "item" @ LayerMember : DeclarationIntroducer;
    ITEM_SOURCE_FIELD => "source" @ ItemMember : FieldIntroducer;
    ITEM_RECORD_FIELD => "record" @ ItemMember : FieldIntroducer;
    ITEM_STATE_FIELD => "state" @ ItemMember : FieldIntroducer;
    ITEM_MAPPING_FIELD => "mapping" @ ItemMember : FieldIntroducer;
    ITEM_TEMPLATE_SLOT_FIELD => "template-slot" @ ItemMember : FieldIntroducer;
    ITEM_MODIFIERS_FIELD => "modifiers" @ ItemMember : FieldIntroducer;
    TRACK_PLAYBACK_FIELD => "playback" @ TrackStateMember : FieldIntroducer;
    TRACK_AUDIO_FIELD => "audio" @ TrackStateMember : FieldIntroducer;
    TRACK_ISOLATION_FIELD => "isolation" @ TrackStateMember : FieldIntroducer;
    TRACK_EDITING_FIELD => "editing" @ TrackStateMember : FieldIntroducer;
    ITEM_PLAYBACK_FIELD => "playback" @ ItemStateMember : FieldIntroducer;
    MEDIA_RESOURCE_REFERENCE_KIND => "resource" @ MediaSourceReferenceKind : ReferenceKind;
    SEQUENCE_REFERENCE_KIND => "sequence" @ SequenceSourceReferenceKind : ReferenceKind;
    MULTICAM_REFERENCE_KIND => "multicam" @ MulticamSourceReferenceKind : ReferenceKind;
    TEMPLATE_ACCEPTS_FIELD => "accepts" @ MediaTemplateSlotMember : FieldIntroducer;
    TEMPLATE_FILL_FIELD => "fill" @ MediaTemplateSlotMember : FieldIntroducer;
    TEMPLATE_LABEL_FIELD => "label" @ MediaTemplateSlotMember : FieldIntroducer;
    TEMPLATE_MINIMUM_DURATION_FIELD => "minimum-source-duration" @ MediaTemplateSlotMember : FieldIntroducer;
}
