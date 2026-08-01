use crate::authoring::{ProjectDecl, ProjectSettings};

use super::annotation::annotation;
use super::delivery::delivery;
use super::multicam::declaration as multicam;
use super::resource::resource;
use super::timeline::sequence;
use super::writer::Writer;

pub(super) fn project(writer: &mut Writer, value: &ProjectDecl) {
    writer.block(format!("project {}", value.id.value), |writer| {
        writer.line(format!("entry {};", super::value::reference(&value.entry)));
        if has_settings(&value.settings) {
            writer.blank();
            settings(writer, &value.settings);
        }
        for resource_value in &value.resources {
            writer.blank();
            resource(writer, resource_value);
        }
        for multicam_value in &value.multicams {
            writer.blank();
            multicam(writer, multicam_value);
        }
        for sequence_value in &value.sequences {
            writer.blank();
            sequence(writer, sequence_value);
        }
        for annotation_value in &value.annotations {
            writer.blank();
            annotation(writer, annotation_value);
        }
        for delivery_value in &value.deliveries {
            writer.blank();
            delivery(writer, delivery_value);
        }
    });
}

fn has_settings(value: &ProjectSettings) -> bool {
    value.timebase.is_some()
        || value.canvas.is_some()
        || value.frame_rate.is_some()
        || value.sample_rate.is_some()
}

fn settings(writer: &mut Writer, value: &ProjectSettings) {
    writer.block("settings", |writer| {
        if let Some(value) = &value.timebase {
            writer.line(format!("timebase {};", value.raw));
        }
        if let Some((width, height)) = &value.canvas {
            writer.line(format!("canvas {} by {};", width.raw, height.raw));
        }
        if let Some(value) = &value.frame_rate {
            writer.line(format!("frame-rate {};", value.raw));
        }
        if let Some(value) = &value.sample_rate {
            writer.line(format!("sample-rate {};", value.raw));
        }
    });
}
