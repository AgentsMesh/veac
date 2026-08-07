use super::support::*;
use crate::{canonical::*, resolve};

pub(super) fn sequence_clip(id: &str) -> Clip {
    let mut clip = generated_clip(id, Generator::Transparent, 0);
    clip.source = ClipSource::Sequence {
        sequence_id: SequenceId::new("seq_nested").unwrap(),
    };
    clip.source_mapping = Some(SourceMapping::linear(time(0), Rational::new(1, 1).unwrap()));
    clip
}

pub(super) fn freeze(id: &str, material: &str, source: i64) -> Clip {
    let mut clip = generated_clip(id, Generator::Transparent, 0);
    clip.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new(material).unwrap(),
        source_time: time(source),
    };
    clip
}

pub(super) fn freeze_project(source_time: RationalTime) -> ProjectEnvelope {
    let mut value = project();
    let clip = &mut value.project.sequences[0].tracks[0].clips[0];
    clip.source = ClipSource::FreezeFrame {
        material_id: MaterialId::new("med_video").unwrap(),
        source_time,
    };
    clip.source_mapping = None;
    value
}

pub(super) fn text_clip(caption: bool) -> Clip {
    let id = if caption { "itm_caption" } else { "itm_text" };
    let mut clip = generated_clip(id, Generator::Transparent, 0);
    let style = text_style(FontRef::Material {
        material_id: MaterialId::new("med_font").unwrap(),
    });
    clip.source = if caption {
        ClipSource::Caption {
            text: "caption".into(),
            speaker: None,
            cue: Box::default(),
            style,
        }
    } else {
        ClipSource::Text {
            text: "title".into(),
            style,
        }
    };
    clip.visual = Some(visual_properties());
    clip
}

pub(super) fn family_text_clip() -> Clip {
    let mut clip = generated_clip("itm_family", Generator::Transparent, 0);
    clip.source = ClipSource::Text {
        text: "title".into(),
        style: text_style(FontRef::Family {
            family: "Inter".into(),
        }),
    };
    clip.visual = Some(visual_properties());
    clip
}

pub(super) fn black() -> Color {
    Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    }
}

pub(super) fn assert_code(value: ProjectEnvelope, code: &str) {
    assert!(resolve(&value, None)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|item| item.code == code));
}
