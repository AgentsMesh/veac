use super::*;

pub(crate) fn program() -> CoreProgram {
    let mut core = Builder::new();
    let identity = core.domain(DomainOperationId::Transform, Vec::new());
    let x = core.literal(numeric("length", 10));
    let y = core.literal(numeric("length", -5));
    let translated = core.domain(DomainOperationId::TransformTranslated, vec![identity, x, y]);
    let x = core.literal(numeric("scalar", 2));
    let y = core.literal(numeric("scalar", 3));
    let scaled = core.domain(DomainOperationId::TransformScaled, vec![translated, x, y]);
    let angle = core.literal(numeric("angle", 15));
    let rotated = core.domain(DomainOperationId::TransformRotated, vec![scaled, angle]);
    let x = core.literal(numeric("scalar", 0));
    let y = core.literal(numeric("scalar", 1));
    let anchored = core.domain(DomainOperationId::TransformAnchored, vec![rotated, x, y]);
    let horizontal = core.literal(Value::Bool(true));
    let vertical = core.literal(Value::Bool(false));
    let flipped = core.domain(
        DomainOperationId::TransformFlipped,
        vec![anchored, horizontal, vertical],
    );
    core.finish(flipped)
}

fn numeric(value_type: &str, value: i128) -> Value {
    let primitive = ValueType::parse(value_type)
        .unwrap()
        .as_primitive()
        .unwrap();
    Value::from_numeric(
        primitive,
        crate::program::expression::ExactNumber::integer(value),
    )
    .unwrap()
}
