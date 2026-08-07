use crate::program::expression::{PrimitiveType, ValueType, ValueTypeKind};
use crate::program::DomainType;

type Case = (String, ValueType, bool);

pub(super) fn cases() -> Vec<Case> {
    let scalar = super::scalar();
    let angle: ValueType = PrimitiveType::Angle.into();
    let point = ValueType::domain(DomainType::Point);
    let vector = ValueType::domain(DomainType::Vector);
    let rect = ValueType::domain(DomainType::Rect);
    vec![
        case("visual-position on clip(owner)", point.clone(), false),
        case("visual-scale on clip(owner)", vector.clone(), false),
        case("visual-rotation on clip(owner)", angle.clone(), false),
        case("visual-crop on clip(owner)", rect, false),
        case("visual-opacity on clip(owner)", scalar.clone(), false),
        case("audio-gain on clip(owner)", scalar.clone(), false),
        case("audio-pan on clip(owner)", scalar.clone(), false),
        case(
            "mask-position on clip-mask(owner, index)",
            vector.clone(),
            false,
        ),
        case(
            "mask-scale on clip-mask(owner, index)",
            vector.clone(),
            false,
        ),
        case(
            "mask-rotation on clip-mask(owner, index)",
            angle.clone(),
            false,
        ),
        case(
            "mask-feather on clip-mask(owner, index)",
            scalar.clone(),
            false,
        ),
        case(
            "mask-expansion on clip-mask(owner, index)",
            scalar.clone(),
            false,
        ),
        case("text-position on text(owner)", point, false),
        case("text-scale on text(owner)", vector.clone(), false),
        case("text-rotation on text(owner)", angle.clone(), false),
        case("text-reveal on text(owner)", scalar.clone(), false),
        case(
            "text-highlight-progress on text(owner)",
            scalar.clone(),
            false,
        ),
        case("text-opacity on text(owner)", scalar.clone(), false),
        case(
            "effect-parameter on clip-effect(owner, id, id)",
            scalar.clone(),
            false,
        ),
        case("apply-opacity on apply(owner)", scalar.clone(), true),
        case(
            "mask-position on apply-mask(owner, index)",
            vector.clone(),
            true,
        ),
        case("mask-scale on apply-mask(owner, index)", vector, true),
        case("mask-rotation on apply-mask(owner, index)", angle, true),
        case(
            "mask-feather on apply-mask(owner, index)",
            scalar.clone(),
            true,
        ),
        case(
            "mask-expansion on apply-mask(owner, index)",
            scalar.clone(),
            true,
        ),
        case(
            "effect-parameter on apply-effect(owner, id, id, id)",
            scalar,
            true,
        ),
    ]
}

fn case(source: &str, result: ValueType, apply: bool) -> Case {
    let body = match result.kind() {
        ValueTypeKind::Primitive(PrimitiveType::Scalar) => "0.5",
        ValueTypeKind::Primitive(PrimitiveType::Angle) => "0deg",
        ValueTypeKind::Domain(value) if value == DomainType::Point => "point(0px, 0px)",
        ValueTypeKind::Domain(value) if value == DomainType::Vector => "vector(1.0, 1.0)",
        ValueTypeKind::Domain(value) if value == DomainType::Rect => "rect(0.0, 0.0, 1.0, 1.0)",
        _ => unreachable!("closed Temporal sink result type"),
    };
    (format!("animate {source} {{ {body} }}"), result, apply)
}
