use super::*;

fn transform() -> &'static str {
    r#"transform_2d(
      transform_motion(
        point_constant(point(10px, -5px)),
        vector_constant(vector(2.0, 0.5)), angle_constant(15deg)),
      transform_geometry(vector(0.25, -0.25), flip_horizontal(),
        vector(0.25, 0.75), crop_none()))"#
}

fn visual() -> String {
    format!(
        r#"visual_style(
          visual_layout(placement_absolute(point(0px, 0px)), frame_none(), {}),
          visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()),
          [], color_pipeline_none())"#,
        transform()
    )
}

#[test]
fn transform_algebra_lowers_to_pure_stable_core_operations() {
    let compiled = compile(transform());
    assert_eq!(
        compiled.result_type().as_domain(),
        Some(DomainType::Transform2D)
    );
    let operations = instructions(&compiled)
        .filter(|instruction| {
            matches!(
                instruction.kind(),
                CoreInstructionKind::DomainConstruct { .. }
            )
        })
        .collect::<Vec<_>>();
    for expected in [
        DomainOperationId::TransformMotion,
        DomainOperationId::TransformGeometry,
        DomainOperationId::Transform2d,
    ] {
        assert!(operations.iter().any(|instruction| matches!(
            instruction.kind(), CoreInstructionKind::DomainConstruct { opcode, .. }
                if *opcode == expected.opcode()
        )));
    }
    assert!(operations
        .iter()
        .all(|value| value.metadata().effect() == Effect::Pure));
    assert!(operations
        .iter()
        .all(|value| value.metadata().shape_stage() == Stage::Build));
}

#[test]
fn item_visual_is_a_graph_emit_with_one_complete_style() {
    let source = format!(
        r#"item(
          identifier("card"), item_enabled(), during(0s, 1s),
          source_generated(generator_transparent()), source_timing_native())
          .with_visual({})"#,
        visual()
    );
    let compiled = compile(&source);
    assert_eq!(compiled.result_type().as_domain(), Some(DomainType::Item));
    let operation = instructions(&compiled).last().unwrap();
    assert!(matches!(
        operation.kind(),
        CoreInstructionKind::GraphEmit { opcode, operands }
            if *opcode == DomainOperationId::ItemWithVisual.opcode() && operands.len() == 2
    ));
    assert_eq!(operation.metadata().effect(), Effect::GraphEmit);
    assert_eq!(operation.metadata().shape_stage(), Stage::Build);
}

#[test]
fn transform_surface_rejects_wrong_units_receivers_and_arity() {
    for (source, code) in [
        ("transform_motion()", "EXPRESSION_CALL_ARITY"),
        (
            "transform_motion(point(1px, 2px), vector_constant(vector(1.0, 1.0)), angle_constant(0deg))",
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        (
            "transform_geometry(vector(0.0, 0.0), flip_none(), point(0px, 0px), crop_none())",
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        (
            "canvas(1px, 1px).with_visual(visual_style)",
            "EXPRESSION_UNKNOWN_METHOD",
        ),
    ] {
        let error = compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty())
            .unwrap_err();
        assert_eq!(error.code(), code, "{source}: {}", error.message());
    }
}
