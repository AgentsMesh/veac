use crate::{Deliverable, DeliverableKind, DeliverableTarget, ImageSequencePattern, RenderConfig};

use super::super::{values::is_file_name, Validator};

pub(super) fn render_config(validator: &mut Validator, output: &RenderConfig, path: &str) {
    let visual = output
        .deliverables
        .iter()
        .any(|value| value.kind.requires_raster());
    match (&output.raster, visual) {
        (Some(raster), true) => {
            if !crate::render_geometry_valid(raster.width, raster.height, raster.frame_rate) {
                validator.push(
                    "OUTPUT_GEOMETRY",
                    Some(output.id.to_string()),
                    path,
                    "raster geometry exceeds the untrusted render budget",
                    None,
                );
            }
        }
        (None, true) => validator.value_error("OUTPUT_RASTER_REQUIRED", path, output.id.as_str()),
        (Some(_), false) => validator.value_error("OUTPUT_RASTER_UNUSED", path, output.id.as_str()),
        (None, false) => {}
    }
}

pub(super) fn target(validator: &mut Validator, value: &Deliverable, path: &str) {
    let pairing_valid = matches!(
        (&value.kind, &value.target),
        (
            DeliverableKind::ImageSequence(_),
            DeliverableTarget::ImageSequence { .. }
        ) | (
            DeliverableKind::Video(_)
                | DeliverableKind::CaptionSidecar(_)
                | DeliverableKind::AudioStem(_)
                | DeliverableKind::Scope(_)
                | DeliverableKind::AudioFile(_)
                | DeliverableKind::AnimatedImage(_)
                | DeliverableKind::StillImage(_),
            DeliverableTarget::File { .. }
        ) | (
            DeliverableKind::AdaptivePackage(_),
            DeliverableTarget::Package { .. }
        )
    );
    if !pairing_valid {
        validator.value_error("OUTPUT_TARGET_KIND", path, value.id.as_str());
    }
    let target_valid = match &value.target {
        DeliverableTarget::File { name } => is_file_name(name),
        DeliverableTarget::ImageSequence { pattern } => {
            ImageSequencePattern::parse(pattern).is_some()
        }
        DeliverableTarget::Package { name } => is_file_name(name),
    };
    if !target_valid {
        validator.value_error("OUTPUT_TARGET", path, value.id.as_str());
    }
}
