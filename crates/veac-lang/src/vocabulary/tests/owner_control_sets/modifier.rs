use crate::vocabulary::{control_uses::modifier as c, CanonicalRole as R, GrammarPosition as P};

use super::{assert_complete, assert_group};

#[test]
fn modifier_controls_have_exact_owners_and_roles() {
    let groups = [
        (
            &[c::LAYOUT_PLACEMENT_FIELD, c::LAYOUT_FRAME_FIELD][..],
            P::LayoutModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::ANCHOR_PLACEMENT_AT_FIELD,
                c::ANCHOR_PLACEMENT_INSET_FIELD,
            ],
            P::AnchorPlacementMember,
            R::FieldIntroducer,
        ),
        (
            &[c::ABSOLUTE_PLACEMENT_POSITION_FIELD],
            P::AbsolutePlacementMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::FRAME_WIDTH_FIELD,
                c::FRAME_HEIGHT_FIELD,
                c::FRAME_FIT_FIELD,
            ],
            P::FrameMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::TRANSFORM_POSITION_FIELD,
                c::TRANSFORM_SCALE_FIELD,
                c::TRANSFORM_SHEAR_FIELD,
                c::TRANSFORM_ROTATION_FIELD,
                c::TRANSFORM_ANCHOR_FIELD,
                c::TRANSFORM_CROP_FIELD,
                c::TRANSFORM_FLIP_HORIZONTAL_FIELD,
                c::TRANSFORM_FLIP_VERTICAL_FIELD,
            ],
            P::TransformModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::COMPOSITE_OPACITY_FIELD,
                c::COMPOSITE_Z_INDEX_FIELD,
                c::COMPOSITE_BLEND_FIELD,
            ],
            P::CompositeModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[c::SURFACE_CORNER_RADIUS_FIELD, c::SURFACE_SHADOW_FIELD],
            P::SurfaceModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::MASK_SHAPE_FIELD,
                c::MASK_POSITION_FIELD,
                c::MASK_SCALE_FIELD,
                c::MASK_ROTATION_FIELD,
                c::MASK_FEATHER_FIELD,
                c::MASK_EXPANSION_FIELD,
                c::MASK_INVERT_FIELD,
            ],
            P::MaskModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[c::ROUNDED_MASK_RADIUS_FIELD],
            P::RoundedRectangleMaskMember,
            R::FieldIntroducer,
        ),
        (
            &[c::POLYGON_MASK_POINT_MEMBER],
            P::PolygonMaskMember,
            R::DeclarationIntroducer,
        ),
        (
            &[c::PATH_MASK_POINT_MEMBER],
            P::PathMaskMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::EFFECT_TYPE_FIELD,
                c::EFFECT_ENABLED_FIELD,
                c::EFFECT_RECORD_FIELD,
            ],
            P::EffectModifierMember,
            R::FieldIntroducer,
        ),
        (
            &[c::EFFECT_PARAMETER_MEMBER],
            P::EffectModifierMember,
            R::DeclarationIntroducer,
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
fn repeated_spellings_keep_distinct_modifier_owners() {
    assert_eq!(
        c::TRANSFORM_POSITION_FIELD.word(),
        c::MASK_POSITION_FIELD.word()
    );
    assert_eq!(
        c::POLYGON_MASK_POINT_MEMBER.word(),
        c::PATH_MASK_POINT_MEMBER.word()
    );
    assert_ne!(c::TRANSFORM_POSITION_FIELD, c::MASK_POSITION_FIELD);
    assert_ne!(c::POLYGON_MASK_POINT_MEMBER, c::PATH_MASK_POINT_MEMBER);
}
