use crate::vocabulary::{control_uses::generator as c, CanonicalRole as R, GrammarPosition as P};

use super::{assert_complete, assert_group};

#[test]
fn generator_controls_have_exact_owners_and_roles() {
    let groups = [
        (
            &[c::SOLID_COLOR_FIELD][..],
            P::SolidGeneratorMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::SHAPE_GEOMETRY_FIELD,
                c::SHAPE_FILL_FIELD,
                c::SHAPE_STROKE_FIELD,
            ],
            P::ShapeGeneratorMember,
            R::FieldIntroducer,
        ),
        (
            &[c::RECTANGLE_BOUNDS_FIELD],
            P::RectangleGeometryMember,
            R::FieldIntroducer,
        ),
        (
            &[
                c::ROUNDED_RECTANGLE_BOUNDS_FIELD,
                c::ROUNDED_RECTANGLE_RADIUS_FIELD,
            ],
            P::RoundedRectangleGeometryMember,
            R::FieldIntroducer,
        ),
        (
            &[c::ELLIPSE_BOUNDS_FIELD],
            P::EllipseGeometryMember,
            R::FieldIntroducer,
        ),
        (
            &[c::POLYGON_POINT_MEMBER],
            P::PolygonGeometryMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::PATH_MOVE_MEMBER,
                c::PATH_LINE_MEMBER,
                c::PATH_CLOSE_MEMBER,
            ],
            P::PathGeometryMember,
            R::DeclarationIntroducer,
        ),
        (
            &[c::LINEAR_FROM_FIELD, c::LINEAR_TO_FIELD],
            P::LinearGradientMember,
            R::FieldIntroducer,
        ),
        (
            &[c::LINEAR_GRADIENT_STOP_MEMBER],
            P::LinearGradientMember,
            R::DeclarationIntroducer,
        ),
        (
            &[
                c::RADIAL_GRADIENT_CENTER_FIELD,
                c::RADIAL_GRADIENT_RADIUS_FIELD,
            ],
            P::RadialGradientMember,
            R::FieldIntroducer,
        ),
        (
            &[c::RADIAL_GRADIENT_STOP_MEMBER],
            P::RadialGradientMember,
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
fn gradient_stops_share_a_word_but_not_an_owner_use() {
    assert_eq!(
        c::LINEAR_GRADIENT_STOP_MEMBER.word(),
        c::RADIAL_GRADIENT_STOP_MEMBER.word()
    );
    assert_ne!(
        c::LINEAR_GRADIENT_STOP_MEMBER,
        c::RADIAL_GRADIENT_STOP_MEMBER
    );
}
