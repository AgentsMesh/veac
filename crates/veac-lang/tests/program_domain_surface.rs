use veac_lang::program::expression::{
    compile_expression, CoreInstructionKind, Effect, ExpressionContext, FunctionDefinition, Stage,
    TypeEnvironment, ValueType,
};
use veac_lang::program::{DomainOperationId, DomainOperationRegistry, DomainType};

#[path = "program_domain_surface/audio_caption.rs"]
mod audio_caption;
#[path = "program_domain_surface/effect.rs"]
mod effect;
#[path = "program_domain_surface/relation.rs"]
mod relation;
#[path = "program_domain_surface/resource.rs"]
mod resource;
#[path = "program_domain_surface/transform.rs"]
mod transform;

fn compile(source: &str) -> veac_lang::program::expression::CompiledExpression {
    compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap()
}

fn instructions(
    compiled: &veac_lang::program::expression::CompiledExpression,
) -> impl Iterator<Item = &veac_lang::program::expression::CoreInstruction> {
    compiled
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
}

#[test]
fn free_domain_calls_lower_to_stable_core_opcodes() {
    let compiled = compile("canvas(1920px, 1080px)");
    assert_eq!(
        compiled.result_type(),
        &ValueType::domain(DomainType::Canvas)
    );
    let operation = instructions(&compiled).last().unwrap();
    assert!(matches!(
        operation.kind(),
        CoreInstructionKind::DomainConstruct { opcode, operands }
            if *opcode == DomainOperationId::Canvas.opcode() && operands.len() == 2
    ));
    assert_eq!(operation.metadata().effect(), Effect::Pure);
    assert_eq!(operation.metadata().shape_stage(), Stage::Build);
    assert_eq!(
        compiled.core().domain_opset(),
        DomainOperationRegistry::standard().version()
    );
    assert_eq!(
        compiled.core().domain_registry_digest(),
        DomainOperationRegistry::standard().digest()
    );
}

#[test]
fn fluent_domain_methods_lower_receiver_first_as_graph_emits() {
    let source = r#"
        {
          let state = track_state(track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked());
          let background = item(identifier("background"), item_enabled(), during(0s, 3s),
            source_generated(generator_solid(#223344ff)), source_timing_native());
          let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
            track_routing_default()).with_item(background);
          let timeline = sequence(identifier("main"), "主时间线",
            sequence_settings(canvas(1920px, 1080px), frame_rate(30, 1), 48000))
            .with_layer(layer);
          project(identifier("demo"), project_settings(600))
            .with_sequence(timeline).entry(timeline)
        }
    "#;
    let compiled = compile(source);
    assert_eq!(
        compiled.result_type().as_domain(),
        Some(DomainType::Project)
    );
    let opcodes = instructions(&compiled)
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::DomainConstruct { opcode, .. }
            | CoreInstructionKind::GraphEmit { opcode, .. } => Some(*opcode),
            _ => None,
        })
        .collect::<Vec<_>>();
    for operation in [
        DomainOperationId::GeneratorSolid,
        DomainOperationId::SourceGenerated,
        DomainOperationId::Item,
        DomainOperationId::LayerWithItem,
        DomainOperationId::SequenceWithLayer,
        DomainOperationId::ProjectWithSequence,
        DomainOperationId::ProjectEntry,
    ] {
        assert!(
            opcodes.contains(&operation.opcode()),
            "{}",
            operation.name()
        );
    }
    let last = instructions(&compiled).last().unwrap();
    assert!(matches!(last.kind(), CoreInstructionKind::GraphEmit { .. }));
    assert_eq!(last.metadata().effect(), Effect::GraphEmit);
    assert_eq!(last.metadata().shape_stage(), Stage::Build);
}

#[test]
fn domain_calls_fail_closed_on_arity_type_method_and_internal_name() {
    for (source, code) in [
        ("canvas(1920px)", "EXPRESSION_CALL_ARITY"),
        ("canvas(1920px, 1s)", "EXPRESSION_CALL_ARGUMENT_TYPE"),
        (
            "canvas(1px, 1px).with_item(item)",
            "EXPRESSION_UNKNOWN_METHOD",
        ),
        (
            "project_with_sequence(project, sequence)",
            "EXPRESSION_UNKNOWN_FUNCTION",
        ),
    ] {
        let error =
            compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty())
                .unwrap_err();
        assert_eq!(error.code(), code, "{source}: {}", error.message());
    }
}

#[test]
fn authored_functions_accept_domain_signatures_and_bodies() {
    let source = r#"
        fn timeline() -> Sequence {
          sequence(identifier("main"), "主时间线",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        }
        fn main(context: Context) -> Project {
          let main = timeline();
          project(identifier("generated"), project_settings(600))
            .with_sequence(main).entry(main)
        }
    "#;
    let prepared = veac_lang::program::prepare_source(source).unwrap();
    assert_eq!(
        prepared.entry_function().return_type().as_domain(),
        Some(DomainType::Project)
    );
}

#[test]
fn domain_function_names_are_reserved() {
    let definition = FunctionDefinition::new(
        "canvas",
        Vec::new(),
        ValueType::domain(DomainType::Canvas),
        "{ canvas(1px, 1px) }",
    );
    let error = veac_lang::program::expression::compile_functions(
        &ExpressionContext::empty(),
        &[definition],
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_FUNCTION_NAME");
}
