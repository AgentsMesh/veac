use super::super::use_macro::define_control_uses;

define_control_uses! {
    LAYOUT_PLACEMENT_FIELD => "placement" @ LayoutModifierMember : FieldIntroducer;
    LAYOUT_FRAME_FIELD => "frame" @ LayoutModifierMember : FieldIntroducer;
    ANCHOR_PLACEMENT_AT_FIELD => "at" @ AnchorPlacementMember : FieldIntroducer;
    ANCHOR_PLACEMENT_INSET_FIELD => "inset" @ AnchorPlacementMember : FieldIntroducer;
    ABSOLUTE_PLACEMENT_POSITION_FIELD => "position" @ AbsolutePlacementMember : FieldIntroducer;
    FRAME_WIDTH_FIELD => "width" @ FrameMember : FieldIntroducer;
    FRAME_HEIGHT_FIELD => "height" @ FrameMember : FieldIntroducer;
    FRAME_FIT_FIELD => "fit" @ FrameMember : FieldIntroducer;
    TRANSFORM_POSITION_FIELD => "position" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_SCALE_FIELD => "scale" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_SHEAR_FIELD => "shear" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_ROTATION_FIELD => "rotation" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_ANCHOR_FIELD => "anchor" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_CROP_FIELD => "crop" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_FLIP_HORIZONTAL_FIELD => "flip-horizontal" @ TransformModifierMember : FieldIntroducer;
    TRANSFORM_FLIP_VERTICAL_FIELD => "flip-vertical" @ TransformModifierMember : FieldIntroducer;
    COMPOSITE_OPACITY_FIELD => "opacity" @ CompositeModifierMember : FieldIntroducer;
    COMPOSITE_Z_INDEX_FIELD => "z-index" @ CompositeModifierMember : FieldIntroducer;
    COMPOSITE_BLEND_FIELD => "blend" @ CompositeModifierMember : FieldIntroducer;
    SURFACE_CORNER_RADIUS_FIELD => "corner-radius" @ SurfaceModifierMember : FieldIntroducer;
    SURFACE_SHADOW_FIELD => "shadow" @ SurfaceModifierMember : FieldIntroducer;
    MASK_SHAPE_FIELD => "shape" @ MaskModifierMember : FieldIntroducer;
    MASK_POSITION_FIELD => "position" @ MaskModifierMember : FieldIntroducer;
    MASK_SCALE_FIELD => "scale" @ MaskModifierMember : FieldIntroducer;
    MASK_ROTATION_FIELD => "rotation" @ MaskModifierMember : FieldIntroducer;
    MASK_FEATHER_FIELD => "feather" @ MaskModifierMember : FieldIntroducer;
    MASK_EXPANSION_FIELD => "expansion" @ MaskModifierMember : FieldIntroducer;
    MASK_INVERT_FIELD => "invert" @ MaskModifierMember : FieldIntroducer;
    ROUNDED_MASK_RADIUS_FIELD => "radius" @ RoundedRectangleMaskMember : FieldIntroducer;
    POLYGON_MASK_POINT_MEMBER => "point" @ PolygonMaskMember : DeclarationIntroducer;
    PATH_MASK_POINT_MEMBER => "point" @ PathMaskMember : DeclarationIntroducer;
    EFFECT_TYPE_FIELD => "type" @ EffectModifierMember : FieldIntroducer;
    EFFECT_ENABLED_FIELD => "enabled" @ EffectModifierMember : FieldIntroducer;
    EFFECT_RECORD_FIELD => "record" @ EffectModifierMember : FieldIntroducer;
    EFFECT_PARAMETER_MEMBER => "parameter" @ EffectModifierMember : DeclarationIntroducer;
}
