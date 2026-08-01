use crate::authoring::{ArtifactRecipe, Diagnostic, Identifier, LayerKind, ProjectDecl, Span};
use std::collections::HashSet;

use super::validate_numbers::{dimension, frame_rate};

pub(super) fn validate(diagnostics: &mut Vec<Diagnostic>, project: &ProjectDecl) {
    let mut artifact_ids = HashSet::new();
    for delivery in &project.deliveries {
        validate_raster(diagnostics, delivery);
        let Some(sequence) = project
            .sequences
            .iter()
            .find(|value| value.id.value == delivery.sequence.value)
        else {
            continue;
        };
        for artifact in &delivery.artifacts {
            if !artifact_ids.insert(artifact.id.value.as_str()) {
                error(
                    diagnostics,
                    "AUTHORING_DUPLICATE_ID",
                    "duplicate artifact id",
                    artifact.id.span,
                );
            }
            super::validate_delivery_extended::artifact(diagnostics, artifact, sequence);
            if let ArtifactRecipe::CaptionSidecar(value) = &artifact.recipe {
                caption_tracks(diagnostics, sequence, &value.track_ids, artifact.span)
            }
        }
    }
}

fn validate_raster(diagnostics: &mut Vec<Diagnostic>, delivery: &crate::authoring::DeliveryDecl) {
    let visual = delivery
        .artifacts
        .iter()
        .any(|value| value.recipe.requires_raster());
    match (&delivery.raster, visual) {
        (Some(raster), true) => {
            dimension(diagnostics, "delivery canvas width", &raster.width);
            dimension(diagnostics, "delivery canvas height", &raster.height);
            frame_rate(diagnostics, &raster.frame_rate);
        }
        (None, true) => error(
            diagnostics,
            "AUTHORING_DELIVERY_RASTER_REQUIRED",
            "delivery with visual artifacts requires raster",
            delivery.span,
        ),
        (Some(raster), false) => error(
            diagnostics,
            "AUTHORING_DELIVERY_RASTER_UNUSED",
            "delivery without visual artifacts must omit raster",
            raster.span,
        ),
        (None, false) => {}
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
            "AUTHORING_DELIVERY_CAPTION_TRACKS_EMPTY",
            "caption sidecar requires at least one caption track",
            span,
        );
    }
    let mut seen = HashSet::new();
    for track in tracks {
        if !seen.insert(track.value.as_str()) {
            error(
                diagnostics,
                "AUTHORING_DELIVERY_CAPTION_TRACK_DUPLICATE",
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
                "AUTHORING_DELIVERY_CAPTION_TRACK_KIND",
                "caption sidecar tracks must reference caption layers",
                track.span,
            ),
            None => super::validate_delivery_extended::missing(diagnostics, track),
        }
    }
}

fn error(diagnostics: &mut Vec<Diagnostic>, code: &'static str, message: &str, span: Span) {
    diagnostics.push(Diagnostic {
        code,
        message: message.to_owned(),
        span,
    });
}
