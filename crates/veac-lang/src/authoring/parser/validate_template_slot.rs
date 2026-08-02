use crate::authoring::{
    Diagnostic, LayerKind, ProjectDecl, ResourceKind, SourceDecl, Span, TemplateMediaKindDecl,
    TemplateSlotDecl,
};
use std::collections::HashMap;

pub(super) fn validate(diagnostics: &mut Vec<Diagnostic>, project: &ProjectDecl) {
    let resources: HashMap<_, _> = project
        .resources
        .iter()
        .map(|value| (value.id.value.as_str(), value.kind))
        .collect();
    let mut uses = HashMap::new();
    for sequence in &project.sequences {
        for layer in &sequence.layers {
            for item in &layer.items {
                if let SourceDecl::Media { resource, .. } = &item.source {
                    *uses.entry(resource.id.value.as_str()).or_insert(0usize) += 1;
                }
            }
        }
    }
    for sequence in &project.sequences {
        for layer in &sequence.layers {
            for item in &layer.items {
                let Some(slot) = &item.template_slot else {
                    continue;
                };
                validate_slot(
                    diagnostics,
                    layer.kind,
                    &item.source,
                    slot,
                    &resources,
                    &uses,
                );
            }
        }
    }
}

fn validate_slot(
    diagnostics: &mut Vec<Diagnostic>,
    layer: LayerKind,
    source: &SourceDecl,
    slot: &TemplateSlotDecl,
    resources: &HashMap<&str, ResourceKind>,
    uses: &HashMap<&str, usize>,
) {
    if !matches!(layer, LayerKind::Video | LayerKind::Visual) {
        error(
            diagnostics,
            "AUTHORING_TEMPLATE_SLOT_LAYER",
            "template slots require a visual layer",
            span(slot),
        );
    }
    match slot {
        TemplateSlotDecl::Text { span } if !matches!(source, SourceDecl::Text { .. }) => error(
            diagnostics,
            "AUTHORING_TEMPLATE_TEXT_SOURCE",
            "text template slots require a text source",
            *span,
        ),
        TemplateSlotDecl::Media {
            accepts,
            label,
            minimum_source_duration,
            span,
            ..
        } => {
            if label.value.trim().is_empty() || label.value.len() > 256 {
                error(
                    diagnostics,
                    "AUTHORING_TEMPLATE_SLOT_LABEL",
                    "template slot label must contain 1..=256 bytes",
                    label.span,
                );
            }
            if minimum_source_duration.is_some() && accepts.value != TemplateMediaKindDecl::Video {
                error(
                    diagnostics,
                    "AUTHORING_TEMPLATE_SLOT_MIN_KIND",
                    "minimum-source-duration is only valid for video slots",
                    minimum_source_duration.as_ref().unwrap().span,
                );
            }
            if let Some(value) = minimum_source_duration {
                super::validate_numbers::time(diagnostics, "minimum-source-duration", value, true);
            }
            let SourceDecl::Media { resource, .. } = source else {
                error(
                    diagnostics,
                    "AUTHORING_TEMPLATE_MEDIA_SOURCE",
                    "media template slots require a media resource source",
                    *span,
                );
                return;
            };
            if resources
                .get(resource.id.value.as_str())
                .is_some_and(|kind| !accepts_kind(accepts.value, *kind))
            {
                error(
                    diagnostics,
                    "AUTHORING_TEMPLATE_MEDIA_KIND",
                    "template slot does not accept its placeholder resource kind",
                    resource.span,
                );
            }
            if uses.get(resource.id.value.as_str()).copied() != Some(1) {
                error(
                    diagnostics,
                    "AUTHORING_TEMPLATE_SLOT_OWNERSHIP",
                    "a media slot placeholder resource must be owned by exactly one item",
                    resource.span,
                );
            }
        }
        TemplateSlotDecl::Text { .. } => {}
    }
}

fn accepts_kind(slot: TemplateMediaKindDecl, resource: ResourceKind) -> bool {
    matches!(
        (slot, resource),
        (TemplateMediaKindDecl::Video, ResourceKind::Video)
            | (TemplateMediaKindDecl::Image, ResourceKind::Image)
            | (
                TemplateMediaKindDecl::VideoOrImage,
                ResourceKind::Video | ResourceKind::Image
            )
    )
}

fn span(value: &TemplateSlotDecl) -> Span {
    match value {
        TemplateSlotDecl::Media { span, .. } | TemplateSlotDecl::Text { span } => *span,
    }
}

fn error(diagnostics: &mut Vec<Diagnostic>, code: &'static str, message: &str, span: Span) {
    diagnostics.push(Diagnostic {
        code,
        message: message.to_owned(),
        span,
    });
}
