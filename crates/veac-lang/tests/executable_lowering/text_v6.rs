use veac_ir::{Clip, ClipSource, ProjectEnvelope, TextStyle};
use veac_lang::program::{
    build_with_loader, BuiltProgram, Diagnostic, Diagnostics, LoadedSource, SourceLoader,
};

pub(super) const SOURCE: &str =
    include_str!("../../../../examples/executable-text-family/main.veac");
const PREVIEW_SOURCE: &str =
    include_str!("../../../../examples/executable-text-family/preview.veac");

#[path = "text_v6/animation.rs"]
mod animation;
#[path = "text_v6/budget_errors.rs"]
mod budget_errors;
#[path = "text_v6/errors.rs"]
mod errors;
#[path = "text_v6/example.rs"]
mod example;
#[path = "text_v6/example_animation.rs"]
mod example_animation;
#[path = "text_v6/variants.rs"]
mod variants;

pub(super) fn envelope() -> ProjectEnvelope {
    envelope_source(SOURCE)
}

pub(super) fn envelope_source(source: &str) -> ProjectEnvelope {
    build(source)
        .unwrap_or_else(|errors| panic!("unexpected diagnostics: {errors}"))
        .envelope()
        .clone()
}

pub(super) fn build(source: &str) -> Result<BuiltProgram, Diagnostics> {
    build_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: source.to_owned(),
        },
        &PreviewLoader,
    )
}

pub(super) fn clips(value: &ProjectEnvelope) -> Vec<&Clip> {
    value.project.sequences[0]
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter(|clip| {
            matches!(
                clip.source,
                ClipSource::Text { .. } | ClipSource::Caption { .. }
            )
        })
        .collect()
}

pub(super) fn style(source: &ClipSource) -> &TextStyle {
    match source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => style,
        _ => panic!("expected typed text source"),
    }
}

pub(super) fn assert_text_error(source: &str) {
    let error = diagnostic(source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(error.message.contains("EXECUTABLE_LOWER_TEXT"), "{error:?}");
}

pub(super) fn assert_animation_error(source: &str) {
    let error = diagnostic(source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(
        error.message.contains("EXECUTABLE_LOWER_ANIMATION"),
        "{error:?}"
    );
}

pub(super) fn assert_time_error(source: &str) {
    let error = diagnostic(source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(error.message.contains("EXECUTABLE_LOWER_TIME"), "{error:?}");
}

fn diagnostic(source: &str) -> Diagnostic {
    build(source)
        .expect_err("source should fail executable lowering")
        .as_slice()
        .first()
        .expect("one bounded diagnostic")
        .clone()
}

struct PreviewLoader;

impl SourceLoader for PreviewLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        if importer == "main.veac" && requested == "./preview.veac" {
            return Ok(LoadedSource {
                id: "preview.veac".to_owned(),
                source: PREVIEW_SOURCE.to_owned(),
            });
        }
        Err(format!(
            "fixture cannot resolve module `{requested}` imported by `{importer}`"
        ))
    }
}
