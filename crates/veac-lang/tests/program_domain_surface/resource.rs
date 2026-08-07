use super::*;

const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn resource_functions_lower_to_graph_emit_and_pure_reference_core() {
    let source = format!(
        r#"source_media(image_resource(
          identifier("poster"), resource_file("assets/poster.png"), sha256("{DIGEST}")))"#
    );
    let compiled = compile(&source);
    assert_eq!(
        compiled.result_type(),
        &ValueType::domain(DomainType::Source)
    );
    let domain = instructions(&compiled)
        .filter(|instruction| {
            matches!(
                instruction.kind(),
                CoreInstructionKind::DomainConstruct { .. } | CoreInstructionKind::GraphEmit { .. }
            )
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        domain[2].kind(),
        CoreInstructionKind::GraphEmit { opcode, operands }
            if *opcode == DomainOperationId::ImageResource.opcode() && operands.len() == 3
    ));
    assert_eq!(domain[2].metadata().effect(), Effect::GraphEmit);
    assert_eq!(domain[2].metadata().shape_stage(), Stage::Build);
    assert!(matches!(
        domain[3].kind(),
        CoreInstructionKind::DomainConstruct { opcode, operands }
            if *opcode == DomainOperationId::SourceMedia.opcode() && operands.len() == 1
    ));
    assert_eq!(domain[3].metadata().effect(), Effect::GraphEmit);
}

#[test]
fn resource_method_and_effectful_map_remain_typed() {
    let source = format!(
        r#"{{
          let keys = [identifier("poster"), identifier("cover")];
          let resources = map(keys, fn(key: identifier) -> Resource effect emit {{
            image_resource(key, resource_file("assets/poster.png"), sha256("{DIGEST}"))
          }});
          let retained_resources = resources;
          project(identifier("demo"), project_settings(600)).with_resource(
            image_resource(identifier("single"), resource_file("assets/poster.png"),
              sha256("{DIGEST}")))
        }}"#
    );
    let compiled = compile(&source);
    assert_eq!(
        compiled.result_type(),
        &ValueType::domain(DomainType::Project)
    );
    let updates = instructions(&compiled)
        .filter(|instruction| {
            matches!(
                instruction.kind(), CoreInstructionKind::GraphEmit { opcode, .. }
                    if *opcode == DomainOperationId::ProjectWithResource.opcode()
            )
        })
        .count();
    assert_eq!(updates, 1);
}

#[test]
fn resource_surface_fails_closed_on_wrong_shapes_and_internal_names() {
    for (source, code) in [
        (
            r#"image_resource(identifier("poster"), resource_file("poster.png"))"#.to_owned(),
            "EXPRESSION_CALL_ARITY",
        ),
        (
            format!(r#"image_resource(identifier("poster"), 1, sha256("{DIGEST}"))"#),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        (
            r#"source_media(generator_transparent())"#.to_owned(),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        (
            format!(
                r#"canvas(1px, 1px).with_resource(image_resource(identifier("x"), resource_file("x.png"), sha256("{DIGEST}")))"#
            ),
            "EXPRESSION_UNKNOWN_METHOD",
        ),
        (
            "project_with_resource(project, resource)".to_owned(),
            "EXPRESSION_UNKNOWN_FUNCTION",
        ),
    ] {
        let error = compile_expression(
            &source,
            &TypeEnvironment::new(),
            &ExpressionContext::empty(),
        )
        .unwrap_err();
        assert_eq!(error.code(), code, "{source}: {}", error.message());
    }
}
