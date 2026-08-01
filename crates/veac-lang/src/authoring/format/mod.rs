mod annotation;
mod annotation_payload;
mod audio_processor;
mod color_stage;
mod delivery;
mod delivery_extended;
mod generator;
mod interpolation;
mod mapping;
mod modifier;
mod modifier_audio;
mod modifier_color;
mod modifier_mask;
mod modifier_surface;
mod multicam;
mod output_aux;
mod output_units;
mod output_video;
mod parameter;
mod project;
mod relation;
mod resource;
mod shadow;
mod structure;
mod template_slot;
mod text;
mod text_animation;
mod text_decor;
mod text_layout;
mod timeline;
mod timeline_state;
mod value;
mod writer;

use crate::authoring::Document;

pub fn format_document(document: &Document) -> String {
    let mut writer = writer::Writer::default();
    project::project(&mut writer, &document.project);
    writer.finish()
}
