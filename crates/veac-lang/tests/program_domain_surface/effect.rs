use super::*;

#[test]
fn plugin_descriptor_and_typed_application_lower_to_closed_core_operations() {
    let compiled = compile(
        r#"video_plugin_scalar_effect(
          identifier("mono"), effect_enabled(effect_window_full()),
          plugin_reference_monochrome_v1(), scalar_constant(0.75)
        )"#,
    );
    assert_eq!(
        compiled.result_type(),
        &ValueType::domain(DomainType::Effect)
    );
    let operations = instructions(&compiled)
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::DomainConstruct { opcode, .. } => Some((opcode, instruction)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let descriptor = operations
        .iter()
        .find(|(opcode, _)| **opcode == DomainOperationId::PluginReferenceMonochromeV1.opcode())
        .unwrap()
        .1;
    assert_eq!(
        compiled
            .core()
            .value_type(descriptor.type_id())
            .and_then(ValueType::as_domain),
        Some(DomainType::PluginEffectDescriptor)
    );
    let plugin = operations
        .iter()
        .find(|(opcode, _)| **opcode == DomainOperationId::VideoPluginScalarEffect.opcode())
        .unwrap()
        .1;
    assert_eq!(plugin.metadata().effect(), Effect::Pure);
    assert_eq!(plugin.metadata().shape_stage(), Stage::Build);
}

#[test]
fn plugin_application_rejects_untyped_descriptor_substitutes() {
    let error = compile_expression(
        r#"video_plugin_scalar_effect(
          identifier("mono"), effect_enabled(effect_window_full()),
          effect_window_full(), scalar_constant(0.5)
        )"#,
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_CALL_ARGUMENT_TYPE");
}
