use super::*;

const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn operation(
    compiled: &veac_lang::program::expression::CompiledExpression,
    id: DomainOperationId,
) -> &veac_lang::program::expression::CoreInstruction {
    instructions(compiled)
        .find(|instruction| match instruction.kind() {
            CoreInstructionKind::DomainConstruct { opcode, .. }
            | CoreInstructionKind::GraphEmit { opcode, .. } => *opcode == id.opcode(),
            _ => false,
        })
        .unwrap_or_else(|| panic!("missing {}", id.name()))
}

#[test]
fn audio_caption_surface_lowers_current_typed_constructors() {
    let source = format!(
        r#"{{
          let audio = audio_resource(identifier("audio"), resource_file("audio.wav"),
            sha256("{DIGEST}"), stream_auto());
          let font = font_resource(identifier("font"), resource_file("font.ttf"),
            sha256("{DIGEST}"));
          let fonts = font_stack(font_resource_ref(font), []);
          let metrics = text_metrics(
            fonts, weight_bold(), font_style_normal(), 48px, 0px, 1.0, #ffffffff);
          let layout = text_layout(
            text_box_auto(), text_wrap_none(), text_overflow_visible(),
            text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed());
          let style = text_style(metrics, layout, text_path_none(),
            text_decoration(text_background_none(), text_outline_none(), shadow_none()),
            [], text_animation_none());
          let sound = item(identifier("sound"), item_enabled(), during(0s, 2s),
            source_media(audio), source_timing_native());
          let spoken = item(identifier("spoken"), item_enabled(), during(0s, 2s),
            source_caption_speaker("欢迎", "Ada", style), source_timing_native());
          [sound, spoken]
        }}"#
    );
    let compiled = compile(&source);
    for id in [
        DomainOperationId::Sha256,
        DomainOperationId::ResourceFile,
        DomainOperationId::AudioResource,
        DomainOperationId::FontResource,
        DomainOperationId::SourceMedia,
        DomainOperationId::SourceCaptionSpeaker,
        DomainOperationId::TextStyle,
        DomainOperationId::Item,
    ] {
        operation(&compiled, id);
    }
    assert_eq!(
        operation(&compiled, DomainOperationId::AudioResource)
            .metadata()
            .effect(),
        Effect::GraphEmit
    );
    assert_eq!(
        operation(&compiled, DomainOperationId::SourceCaptionSpeaker)
            .metadata()
            .effect(),
        Effect::GraphEmit
    );
}

#[test]
fn audio_caption_surface_rejects_wrong_arity_units_and_shapes() {
    let identity = format!(r#"sha256("{DIGEST}")"#);
    for (source, code) in [
        (
            r#"audio_resource(identifier("a"), resource_file("a.wav"), sha256("00"))"#.to_owned(),
            "EXPRESSION_CALL_ARITY",
        ),
        (
            r#"font_resource(identifier("f"), "f.ttf", sha256("00"))"#.to_owned(),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        (
            format!(
                r#"source_media(image_resource(identifier("i"), resource_file("i.png"), {identity})).with_audio(1s)"#
            ),
            "EXPRESSION_UNKNOWN_METHOD",
        ),
        (
            r#"source_caption_speaker("字", 1, text_style)"#.to_owned(),
            "EXPRESSION_CALL_ARGUMENT_TYPE",
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

#[test]
fn current_callable_names_are_not_lexer_keywords() {
    let keywords = veac_lang::vocabulary::language_spec()
        .vocabulary
        .lexer_keywords;
    for name in [
        "audio_resource",
        "font_resource",
        "source_caption_speaker",
        "audio_style",
    ] {
        assert!(!keywords.iter().any(|value| value == name));
    }
}
