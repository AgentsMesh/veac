use crate::authoring::{
    AudioStemSourceDecl, Diagnostic, Identifier, LayerKind, OutputDecl, OutputEncoding,
    ProjectDecl, Span,
};
use std::collections::HashSet;

pub(super) fn validate(diagnostics: &mut Vec<Diagnostic>, project: &ProjectDecl) {
    for output in &project.outputs {
        validate_file_name(diagnostics, output);
        let Some(sequence) = project
            .sequences
            .iter()
            .find(|value| value.id.value == output.sequence.value)
        else {
            continue;
        };
        match &output.encoding {
            OutputEncoding::CaptionSidecar(value) => {
                caption_tracks(diagnostics, sequence, &value.track_ids, output.span)
            }
            OutputEncoding::AudioStem(value) => match &value.source {
                AudioStemSourceDecl::Track(source) => match sequence
                    .layers
                    .iter()
                    .find(|value| value.id.value == source.value)
                {
                    Some(layer) if layer.kind == LayerKind::Audio => {}
                    Some(_) => error(
                        diagnostics,
                        "AUTHORING_OUTPUT_AUDIO_TRACK_KIND",
                        "audio stem track source must reference an audio layer",
                        source.span,
                    ),
                    None => missing(diagnostics, source),
                },
                AudioStemSourceDecl::Bus(source) => {
                    let found = sequence.layers.iter().any(|value| {
                        value
                            .route_bus
                            .as_ref()
                            .is_some_and(|bus| bus.value == source.value)
                    });
                    if !found {
                        missing(diagnostics, source);
                    }
                }
                AudioStemSourceDecl::Master => {}
            },
            _ => {}
        }
    }
}

fn caption_tracks(
    diagnostics: &mut Vec<Diagnostic>,
    sequence: &crate::authoring::SequenceDecl,
    tracks: &[Identifier],
    span: Span,
) {
    if tracks.is_empty() {
        error(
            diagnostics,
            "AUTHORING_OUTPUT_CAPTION_TRACKS_EMPTY",
            "caption sidecar requires at least one caption track",
            span,
        );
    }
    let mut seen = HashSet::new();
    for track in tracks {
        if !seen.insert(track.value.as_str()) {
            error(
                diagnostics,
                "AUTHORING_OUTPUT_CAPTION_TRACK_DUPLICATE",
                "caption sidecar track references must be unique",
                track.span,
            );
        }
        match sequence
            .layers
            .iter()
            .find(|value| value.id.value == track.value)
        {
            Some(layer) if layer.kind == LayerKind::Caption => {}
            Some(_) => error(
                diagnostics,
                "AUTHORING_OUTPUT_CAPTION_TRACK_KIND",
                "caption sidecar tracks must reference caption layers",
                track.span,
            ),
            None => missing(diagnostics, track),
        }
    }
}

fn validate_file_name(diagnostics: &mut Vec<Diagnostic>, output: &OutputDecl) {
    let value = &output.file_name.value;
    let invalid = value.is_empty()
        || value.len() > 255
        || matches!(value.as_str(), "." | "..")
        || value
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'));
    if invalid {
        error(
            diagnostics,
            "AUTHORING_OUTPUT_FILE_NAME",
            "output file-name must be a safe basename of at most 255 bytes",
            output.file_name.span,
        );
    }
}

fn missing(diagnostics: &mut Vec<Diagnostic>, value: &Identifier) {
    error(
        diagnostics,
        "AUTHORING_REFERENCE_NOT_FOUND",
        "output reference target does not exist in its selected sequence",
        value.span,
    );
}

fn error(diagnostics: &mut Vec<Diagnostic>, code: &'static str, message: &str, span: Span) {
    diagnostics.push(Diagnostic {
        code,
        message: message.to_owned(),
        span,
    });
}
