use veac_lang::program::expression::{
    compile_expression, BuildInputSlot, CoreBuildInputId, CoreDigest, CoreInputDeclarationDigest,
    CoreTemporalComposeOperation, ExpressionContext, PrimitiveType, TypeEnvironment, UnitSuffix,
};
use veac_lang::program::{
    DomainOperationId, DomainOperationRegistry, DomainOpsetVersion, DomainType, DomainValueShape,
    MethodRegistry,
};
use veac_lang::SyntaxToken;

fn compile(source: &str, types: &TypeEnvironment) -> veac_lang::program::expression::CoreProgram {
    compile_expression(source, types, &ExpressionContext::empty())
        .unwrap()
        .core()
        .clone()
}

#[test]
fn verified_core_public_model_is_fully_observable() {
    let program = compile("if true { 1 } else { 2 }", &TypeEnvironment::new());
    assert_eq!(
        program.version(),
        veac_lang::program::expression::CORE_VERSION
    );
    assert_eq!(program.domain_opset(), DomainOpsetVersion::CURRENT);
    assert_eq!(
        program.domain_registry_digest(),
        DomainOperationRegistry::standard().digest()
    );
    assert_eq!(
        program.types().entries().len(),
        program.types().entries().len()
    );
    assert!(program.types().get(program.result_type_id()).is_some());
    assert_eq!(
        program.types().value(program.result_type_id()),
        Some(program.result_type())
    );
    assert_eq!(
        program.value_type(program.result_type_id()),
        Some(program.result_type())
    );
    assert!(!program.blocks().is_empty());
    assert!(program.nominal_definitions().is_empty());
    assert!(program.inputs().is_empty());
    assert!(program.closure_definitions().is_empty());
    assert_eq!(program.closure_definition_count(), 0);
    assert_ne!(program.input_declarations_digest().as_bytes(), &[0; 32]);

    for block in program.blocks() {
        let _ = block.id();
        assert_eq!(block.instructions().len(), block.instructions().len());
        assert_eq!(block.parameters().len(), block.parameters().len());
        let _ = block.terminator();
        for parameter in block.parameters() {
            let _ = (
                parameter.id(),
                parameter.type_id(),
                parameter.span(),
                parameter.metadata(),
            );
        }
        for instruction in block.instructions() {
            let _ = (
                instruction.id(),
                instruction.kind(),
                instruction.type_id(),
                instruction.span(),
                instruction.metadata(),
            );
        }
    }
    for entry in program.types().entries() {
        assert_eq!(program.types().get(entry.id()), Some(entry.kind()));
        let _ = (entry.kind().value_type(), entry.kind().is_internal());
    }
}

#[test]
fn core_input_closure_summary_and_digest_accessors_are_live() {
    let types = [("seed".to_owned(), PrimitiveType::Integer.into())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let input_program = compile("seed", &types);
    let input = &input_program.inputs()[0];
    assert_eq!(input.id().value(), 0);
    assert_eq!(input.name(), "seed");
    assert_eq!(input.type_id(), input_program.result_type_id());
    assert!(!input.trusts_function_value());
    assert!(input.callable().is_none());
    assert_eq!(input.span(), 0..4);

    let closure_program = compile(
        "fn(value: int) -> int effect pure { value + 1 }",
        &TypeEnvironment::new(),
    );
    let closure = &closure_program.closure_definitions()[0];
    assert_eq!(closure.id().value(), 0);
    assert_eq!(closure.parameter_types().len(), 1);
    assert!(closure.capture_types().is_empty());
    assert!(!closure.non_escaping());
    assert_eq!(closure.body().result_type(), &PrimitiveType::Integer.into());
    assert_ne!(closure.digest().as_bytes(), &[0; 32]);
    assert!(!closure.span().is_empty());
    let summary = closure.summary();
    assert_eq!(summary.result().effect(), summary.effect());
    assert!(!summary.contains_local_mutation());
    assert!(summary.instruction_count() > 0);

    let digest = CoreDigest::from_bytes([0xab; 32]);
    assert_eq!(digest.to_string(), "ab".repeat(32));
    let inputs = CoreInputDeclarationDigest::from_bytes([0xcd; 32]);
    assert_eq!(inputs.to_string(), "cd".repeat(32));
}

#[test]
fn domain_registry_and_closed_value_shape_accessors_are_live() {
    let registry = DomainOperationRegistry::standard();
    assert_eq!(registry.len(), registry.contracts().len());
    assert!(!registry.is_empty());
    assert!(registry.retained_bytes() > 0);
    let canvas = registry.lookup_name("canvas").unwrap();
    assert_eq!(canvas.id(), DomainOperationId::Canvas);
    assert_eq!(canvas.exposure().receiver(), None);
    let method = registry
        .contracts()
        .find(|contract| contract.exposure().receiver().is_some())
        .unwrap();
    assert!(method.exposure().receiver().is_some());
    assert!(DomainValueShape::domain(DomainType::Canvas).is_single_domain());
    assert!(!DomainValueShape::domain_list(DomainType::Canvas).is_single_domain());
    assert_eq!(format!("{:?}", DomainType::Canvas), "Canvas");
    assert_eq!(DomainOperationId::Canvas.to_string(), "0x1001");

    let error = DomainOperationRegistry::for_version(DomainOpsetVersion::from_raw(0)).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPSET_UNSUPPORTED");
    assert_eq!(error.message(), error.to_string());
    assert!(error.to_string().contains("not supported"));
}

#[test]
fn public_context_units_and_syntax_trait_cover_the_closed_tables() {
    let input = BuildInputSlot::new(
        CoreBuildInputId::for_symbol("seed"),
        "seed".to_owned(),
        PrimitiveType::Integer.into(),
    );
    let context = ExpressionContext::empty()
        .with_methods(MethodRegistry::default())
        .with_build_inputs([("seed".to_owned(), input)].into_iter().collect());
    assert!(context.types().is_empty());
    assert!(context.functions().is_empty());
    assert!(context.methods().is_empty());
    assert!(!context.domain().is_empty());
    assert_eq!(context.build_input("seed").unwrap().name(), "seed");
    assert_eq!(context.build_inputs().count(), 1);

    for unit in UnitSuffix::ALL {
        assert_eq!(
            <UnitSuffix as SyntaxToken>::parse(unit.as_str()),
            Some(unit)
        );
        assert_eq!(<UnitSuffix as SyntaxToken>::as_str(unit), unit.as_str());
        let _ = (
            unit.scale(),
            unit.dimension(),
            unit.is_executable_expression(),
        );
    }
    assert_eq!(UnitSuffix::split_literal("120.5ms"), ("120.5", "ms"));
    assert_eq!(UnitSuffix::split_literal("42"), ("42", ""));
    assert_eq!(CoreTemporalComposeOperation::Vec2.arity(), 2);
    assert_eq!(CoreTemporalComposeOperation::Color.arity(), 4);
}
