use veac_lang::program::expression::{
    compile_expression, compile_functions, CollectionOperation, CoreForEach, CoreInstruction,
    CoreInstructionKind, CoreTerminator, Effect, ExpressionContext, FunctionDefinition,
    FunctionParameter, PrimitiveType, Stage, TypeEnvironment, ValueType,
};

fn compile(source: &str) -> veac_lang::program::expression::CompiledExpression {
    compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap()
}

fn for_each(compiled: &veac_lang::program::expression::CompiledExpression) -> &CoreForEach {
    compiled
        .core()
        .blocks()
        .iter()
        .find_map(|block| match block.terminator() {
            CoreTerminator::ForEach(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

fn collection(compiled: &veac_lang::program::expression::CompiledExpression) -> &CoreInstruction {
    compiled
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| {
            matches!(
                instruction.kind(),
                CoreInstructionKind::Collection {
                    operation: CollectionOperation::Map,
                    ..
                }
            )
        })
        .unwrap()
}

#[test]
fn map_and_surface_for_admit_verified_graph_emit_callbacks() {
    for generated in [
        r#"map(keys, fn(key: identifier) -> Item effect emit {
            item(key, item_enabled(), during(0s, 1s),
              source_generated(generator_solid(#223344ff)), source_timing_native()) })"#,
        r#"for key in keys { item(key, item_enabled(), during(0s, 1s),
            source_generated(generator_solid(#223344ff)), source_timing_native()) }"#,
    ] {
        let source = format!(
            r#"{{
                let keys = [identifier("first"), identifier("second")];
                let sources = {generated};
                let retained_sources = sources;
                let timeline = sequence(identifier("main"), "图发射迭代",
                    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
                project(identifier("demo"), project_settings(600))
                    .with_sequence(timeline).entry(timeline)
            }}"#
        );
        let compiled = compile(&source);
        let metadata = if generated.starts_with("map") {
            collection(&compiled).metadata()
        } else {
            for_each(&compiled).result_metadata()
        };
        assert_eq!(metadata.effect(), Effect::GraphEmit);
        assert_eq!(metadata.shape_stage(), Stage::Const);
        assert_eq!(metadata.leaf_stage(), Stage::Build);
        let final_value = compiled
            .core()
            .blocks()
            .iter()
            .flat_map(|block| block.instructions())
            .last()
            .unwrap();
        assert_eq!(final_value.metadata().effect(), Effect::GraphEmit);
        assert_eq!(final_value.metadata().shape_stage(), Stage::Build);
    }
}

#[test]
fn imported_graph_emit_helpers_remain_visible_through_map_callbacks() {
    let helper = FunctionDefinition::new(
        "build_item",
        vec![FunctionParameter::new(
            "key",
            ValueType::primitive(PrimitiveType::Identifier),
        )],
        ValueType::domain(veac_lang::program::DomainType::Item),
        "{ item(key, item_enabled(), during(0s, 1s), \
         source_generated(generator_solid(#223344ff)), source_timing_native()) }",
    );
    let context = compile_functions(&ExpressionContext::empty(), &[helper]).unwrap();
    let compiled = compile_expression(
        r#"map([identifier("first")], fn(key: identifier) -> Item effect emit {
            build_item(key)
        })"#,
        &TypeEnvironment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(collection(&compiled).metadata().effect(), Effect::GraphEmit);
}

#[test]
fn surface_for_resolves_a_captured_graph_emit_closure() {
    let compiled = compile(
        r#"{
            let emit = fn(key: identifier) -> Item effect emit {
                item(key, item_enabled(), during(0s, 1s),
                  source_generated(generator_solid(#223344ff)), source_timing_native())
            };
            for key in [identifier("first")] { emit(key) }
        }"#,
    );
    assert_eq!(
        for_each(&compiled).result_metadata().effect(),
        Effect::GraphEmit
    );
}

#[test]
fn surface_for_retains_lexical_binding_provenance_in_core() {
    let source = "for authored_key in [identifier(\"first\")] { authored_key }";
    let compiled = compile(source);
    let binding = for_each(&compiled).provenance().binding_span().clone();
    assert_eq!(&source[binding], "authored_key");

    let map = compile("map([1], fn(value: int) -> int effect pure { value })");
    collection(&map);
    assert!(map
        .core()
        .blocks()
        .iter()
        .all(|block| !matches!(block.terminator(), CoreTerminator::ForEach(_))));
}

#[test]
fn filter_and_fold_still_reject_graph_emit_callbacks() {
    for source in [
        r#"filter([identifier("a")], fn(key: identifier) -> bool effect emit {
            let emitted = item(key, item_enabled(), during(0s, 1s),
              source_generated(generator_solid(#223344ff)), source_timing_native()); true
        })"#,
        r#"fold([identifier("a")], 0, fn(total: int, key: identifier) -> int effect emit {
            let emitted = item(key, item_enabled(), during(0s, 1s),
              source_generated(generator_solid(#223344ff)), source_timing_native()); total + 1
        })"#,
    ] {
        let error =
            compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty())
                .unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_COLLECTION_CALLBACK_EFFECT");
        assert!(error.message().contains("does not accept an `effect emit`"));
    }
}
