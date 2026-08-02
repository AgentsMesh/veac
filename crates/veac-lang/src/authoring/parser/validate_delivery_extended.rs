use std::collections::BTreeSet;

use crate::authoring::{
    ArtifactDecl, ArtifactRecipe, ArtifactTargetDecl, AudioMixSourceDecl, Diagnostic, Identifier,
    LayerKind, SequenceDecl, Span,
};

pub(super) fn artifact(
    diagnostics: &mut Vec<Diagnostic>,
    value: &ArtifactDecl,
    sequence: &SequenceDecl,
) {
    target(diagnostics, value);
    match &value.recipe {
        ArtifactRecipe::AudioStem(encoding) => source(diagnostics, sequence, &encoding.source),
        ArtifactRecipe::AudioFile(encoding) => source(diagnostics, sequence, &encoding.source),
        ArtifactRecipe::AdaptivePackage(package) => {
            if let Some(audio) = &package.audio {
                source(diagnostics, sequence, &audio.source);
            }
            rendition_ids(diagnostics, package);
        }
        _ => {}
    }
}

fn rendition_ids(
    diagnostics: &mut Vec<Diagnostic>,
    package: &crate::authoring::AdaptivePackageRecipe,
) {
    let mut seen = BTreeSet::new();
    for rendition in &package.renditions {
        if !seen.insert(rendition.id.value.as_str()) {
            error(
                diagnostics,
                "AUTHORING_HLS_RENDITION_DUPLICATE",
                "HLS rendition identifiers must be unique",
                rendition.id.span,
            );
        }
    }
}

fn target(diagnostics: &mut Vec<Diagnostic>, value: &ArtifactDecl) {
    let paired = matches!(
        (&value.recipe, &value.target),
        (
            ArtifactRecipe::ImageSequence(_),
            ArtifactTargetDecl::ImageSequence(_)
        ) | (
            ArtifactRecipe::AdaptivePackage(_),
            ArtifactTargetDecl::Package(_)
        ) | (
            ArtifactRecipe::Video(_)
                | ArtifactRecipe::CaptionSidecar(_)
                | ArtifactRecipe::AudioStem(_)
                | ArtifactRecipe::Scope(_)
                | ArtifactRecipe::AudioFile(_)
                | ArtifactRecipe::AnimatedImage(_)
                | ArtifactRecipe::StillImage(_),
            ArtifactTargetDecl::File(_)
        )
    );
    if !paired {
        error(
            diagnostics,
            "AUTHORING_ARTIFACT_TARGET_KIND",
            "artifact kind and target kind do not match",
            value.target.value().span,
        );
    }
    if let ArtifactTargetDecl::ImageSequence(pattern) = &value.target {
        if veac_ir::ImageSequencePattern::parse(&pattern.value).is_none() {
            error(
                diagnostics,
                "AUTHORING_ARTIFACT_TARGET_PATTERN",
                "pattern target must contain one safe printf frame placeholder",
                pattern.span,
            );
        }
    }
}

fn source(diagnostics: &mut Vec<Diagnostic>, sequence: &SequenceDecl, value: &AudioMixSourceDecl) {
    match value {
        AudioMixSourceDecl::Track(id) => match sequence
            .layers
            .iter()
            .find(|layer| layer.id.value == id.value)
        {
            Some(layer) if matches!(layer.kind, LayerKind::Video | LayerKind::Audio) => {}
            Some(_) => error(
                diagnostics,
                "AUTHORING_DELIVERY_AUDIO_TRACK_KIND",
                "audio source track must reference a video or audio layer",
                id.span,
            ),
            None => missing(diagnostics, id),
        },
        AudioMixSourceDecl::Bus(id) => {
            if !sequence.layers.iter().any(|layer| {
                layer
                    .route_bus
                    .as_ref()
                    .is_some_and(|bus| bus.value == id.value)
            }) {
                missing(diagnostics, id);
            }
        }
        AudioMixSourceDecl::Master => {}
    }
}

pub(super) fn missing(diagnostics: &mut Vec<Diagnostic>, value: &Identifier) {
    error(
        diagnostics,
        "AUTHORING_REFERENCE_NOT_FOUND",
        "artifact reference target does not exist in its delivery sequence",
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
