mod aux;
mod collision;
mod contract;

use crate::*;

use super::{output_file_compatible, Validator};

impl Validator {
    pub(super) fn render_config(&mut self, output: &RenderConfig, project: &Project) {
        let path = format!("/project/render_configs/{}", output.id);
        self.check_id(output.id.is_valid(), output.id.as_str(), &path);
        if !self.output_ids.insert(output.id.to_string()) {
            self.duplicate("DUPLICATE_OUTPUT_ID", output.id.as_str(), &path);
        }
        if !self.sequence_ids.contains(output.sequence_id.as_str()) {
            self.missing_ref(
                "OUTPUT_SEQUENCE_NOT_FOUND",
                output.sequence_id.as_str(),
                &format!("{path}/sequence_id"),
            );
        }
        contract::render_config(self, output, &path);
        if output.deliverables.is_empty() {
            self.value_error("OUTPUT_DELIVERABLE_REQUIRED", &path, output.id.as_str());
        }
        let mut previous: Option<&str> = None;
        let mut files: Vec<&Deliverable> = Vec::new();
        for deliverable in &output.deliverables {
            let item_path = format!("{path}/deliverables/{}", deliverable.id);
            self.check_id(
                deliverable.id.is_valid(),
                deliverable.id.as_str(),
                &item_path,
            );
            if previous.is_some_and(|id| id >= deliverable.id.as_str()) {
                self.value_error(
                    "OUTPUT_DELIVERABLE_ORDER",
                    &item_path,
                    deliverable.id.as_str(),
                );
            }
            previous = Some(deliverable.id.as_str());
            if files
                .iter()
                .any(|existing| collision::overlap(existing, deliverable))
            {
                self.value_error("OUTPUT_FILE_COLLISION", &item_path, deliverable.id.as_str());
            }
            files.push(deliverable);
            if !self.deliverable_ids.insert(deliverable.id.to_string()) {
                self.duplicate(
                    "DUPLICATE_DELIVERABLE_ID",
                    deliverable.id.as_str(),
                    &item_path,
                );
            }
            contract::target(self, deliverable, &item_path);
            self.deliverable(deliverable, output, project, &item_path);
        }
    }

    fn deliverable(
        &mut self,
        value: &Deliverable,
        output: &RenderConfig,
        project: &Project,
        path: &str,
    ) {
        match &value.kind {
            DeliverableKind::Video(settings) => {
                self.video_deliverable(value, settings, output, path)
            }
            _ => self.aux_deliverable(value, project, output, path),
        }
    }

    fn video_deliverable(
        &mut self,
        value: &Deliverable,
        settings: &VideoDeliverable,
        output: &RenderConfig,
        path: &str,
    ) {
        let pixel_valid = output.raster.as_ref().is_none_or(|raster| {
            pixel_geometry_valid(raster.width, raster.height, settings.video.pixel_format)
        });
        if !video_settings_valid(&settings.video) || !video_delivery_valid(settings) || !pixel_valid
        {
            self.value_error("OUTPUT_VIDEO_SETTINGS", path, value.id.as_str());
        }
        if !video_container_compatible(settings.container, settings.video.codec) {
            self.value_error("OUTPUT_VIDEO_CODEC", path, value.id.as_str());
        }
        if !value
            .target
            .file_name()
            .is_some_and(|name| output_file_compatible(name, settings.container))
        {
            self.value_error("OUTPUT_FILE_FORMAT", path, value.id.as_str());
        }
        if settings.audio.as_ref().is_some_and(|audio| {
            !audio_output_valid(audio)
                || !audio_container_compatible(settings.container, audio.codec)
        }) {
            self.value_error("OUTPUT_AUDIO", path, value.id.as_str());
        }
        if settings.optimize_for_streaming
            && !matches!(settings.container, OutputFormat::Mp4 | OutputFormat::Mov)
        {
            self.value_error("OUTPUT_STREAMING_MODE", path, value.id.as_str());
        }
        if settings.container == OutputFormat::Mxf
            && !output.raster.as_ref().is_some_and(|raster| {
                mxf_geometry_valid(raster.width, raster.height, raster.frame_rate)
            })
        {
            self.value_error("OUTPUT_MXF_SETTINGS", path, value.id.as_str());
        }
    }
}
