use super::{domain_fixture::compiled, empty_registry, raw};
use crate::program::expression::runtime;
use crate::program::expression::{
    CoreInstructionKind, CoreValueMetadata, Environment, ExecutionBudget, ValueId,
};
use crate::program::{DomainOperationId, DomainOpsetVersion, DomainType};

const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn caption_source() -> String {
    format!(
        r#"{{
            let font = font_resource(identifier("font"), "font.ttf", sha256("{DIGEST}"));
            caption_item_with_speaker(
                identifier("caption"), "hello", "Ada", during(0s, 2s),
                text_style(font, #ffffffff, 48px)
            )
        }}"#
    )
}

fn audio_source() -> String {
    format!(
        r#"audio_item(
            identifier("voice"),
            media(audio_resource(identifier("audio"), "audio.wav", sha256("{DIGEST}"))),
            during(0s, 2s)
        )"#
    )
}

fn project_source() -> String {
    format!(
        r#"{{
          let audio = audio_resource(identifier("audio"), "audio.wav", sha256("{DIGEST}"));
          let font = font_resource(identifier("font"), "font.ttf", sha256("{DIGEST}"));
          let sound = audio_item(identifier("sound"), media(audio), during(0s, 2s));
          let caption = caption_item(identifier("caption"), "hello", during(0s, 2s),
              text_style(font, #ffffffff, 48px));
          project(identifier("project"), canvas(640px, 360px), fps(30))
            .with_resources([audio, font])
            .with_sequence(sequence(identifier("main")).with_layers([
              audio_layer(identifier("audio")).with_item(sound),
              caption_layer(identifier("captions")).with_item(caption)
            ])).entry(identifier("main"))
        }}"#
    )
}

fn program(source: &str) -> super::super::CoreProgram {
    raw(source).0
}

fn operation(program: &super::super::CoreProgram, id: DomainOperationId) -> usize {
    program.blocks[0]
        .instructions
        .iter()
        .position(|instruction| match instruction.kind() {
            CoreInstructionKind::DomainConstruct { opcode, .. }
            | CoreInstructionKind::GraphEmit { opcode, .. } => *opcode == id.opcode(),
            _ => false,
        })
        .unwrap()
}

fn failure(program: super::super::CoreProgram) -> String {
    super::super::verify(program, &empty_registry(), &[])
        .expect_err("mutated V5 Core must fail verification")
        .message()
        .to_owned()
}

#[test]
fn raw_audio_caption_core_uses_v5_closed_operations() {
    for (source, expected, operations) in [
        (
            audio_source(),
            DomainType::Item,
            vec![
                DomainOperationId::Sha256,
                DomainOperationId::AudioResource,
                DomainOperationId::Media,
                DomainOperationId::AudioItem,
            ],
        ),
        (
            caption_source(),
            DomainType::Item,
            vec![
                DomainOperationId::Sha256,
                DomainOperationId::FontResource,
                DomainOperationId::TextStyle,
                DomainOperationId::CaptionItemWithSpeaker,
            ],
        ),
    ] {
        let program = program(&source);
        assert_eq!(program.domain_opset(), DomainOpsetVersion::V8);
        assert_eq!(program.result_type().as_domain(), Some(expected));
        for operation_id in operations {
            operation(&program, operation_id);
        }
        compiled(program);
    }
}

#[test]
fn caption_core_rejects_opcode_kind_type_and_metadata_corruption() {
    let mut opcode = program(&caption_source());
    let index = operation(&opcode, DomainOperationId::CaptionItemWithSpeaker);
    let CoreInstructionKind::GraphEmit { opcode: value, .. } =
        &mut opcode.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    *value = 0xffff;
    assert!(failure(opcode).contains("unknown domain operation opcode"));

    let mut kind = program(&caption_source());
    let index = operation(&kind, DomainOperationId::CaptionItemWithSpeaker);
    let CoreInstructionKind::GraphEmit { operands, .. } = &kind.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    kind.blocks[0].instructions[index].kind = CoreInstructionKind::DomainConstruct {
        opcode: DomainOperationId::CaptionItemWithSpeaker.opcode(),
        operands: operands.clone(),
    };
    assert!(failure(kind).contains("wrong Core instruction kind"));

    let mut operand = program(&caption_source());
    let index = operation(&operand, DomainOperationId::CaptionItemWithSpeaker);
    let CoreInstructionKind::GraphEmit { operands, .. } =
        &mut operand.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    operands[2] = ValueId::new(1);
    assert!(failure(operand).contains("operand `speaker` type"));

    let mut metadata = program(&caption_source());
    let index = operation(&metadata, DomainOperationId::CaptionItemWithSpeaker);
    metadata.blocks[0].instructions[index].metadata = CoreValueMetadata::constant();
    assert!(failure(metadata).contains("domain instruction metadata"));
}

#[test]
fn verified_core_runtime_rechecks_resource_and_source_affinity() {
    let source = project_source();
    assert_affinity_failure(
        &source,
        DomainOperationId::AudioResource,
        DomainOperationId::ImageResource,
        "DOMAIN_SOURCE_AFFINITY",
    );
    assert_affinity_failure(
        &source,
        DomainOperationId::FontResource,
        DomainOperationId::ImageResource,
        "DOMAIN_RESOURCE_AFFINITY",
    );
}

fn assert_affinity_failure(
    source: &str,
    original: DomainOperationId,
    replacement: DomainOperationId,
    code: &str,
) {
    let mut program = program(source);
    let index = operation(&program, original);
    let CoreInstructionKind::GraphEmit { opcode, .. } =
        &mut program.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    *opcode = replacement.opcode();
    let expression = compiled(program);
    let error = runtime::execute_project(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), code, "{original:?} -> {replacement:?}");
}
