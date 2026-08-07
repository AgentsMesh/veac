use std::collections::BTreeSet;

use veac_ir::{
    Anchor, Animatable, ClipSource, ColorStage, Generator, Gradient, MaterialKind, Placement,
    ProjectEnvelope, TextGranularity,
};

use crate::support::{assert_preview_evidence, text_styles};

#[path = "text_color/layout.rs"]
mod layout;

#[test]
fn preview_text_and_color_rows_have_typed_evidence_in_their_target() {
    assert_preview_evidence("text-color.json", evidence);
}

fn evidence(envelope: &ProjectEnvelope) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    if crate::support::clips(envelope).any(|clip| {
        matches!(
            &clip.source,
            ClipSource::Text { .. } | ClipSource::Caption { .. }
        ) && clip.visual.as_ref().is_some_and(positioned)
    }) {
        found.insert("text.layout.position".to_owned());
    }
    if crate::support::clips(envelope).any(|clip| {
        matches!(
            &clip.source,
            ClipSource::Text { text, .. }
                if text.contains("中文排版") && text.contains("مرحبا")
        )
    }) {
        found.insert("text.layout.unicode-bidi".to_owned());
    }
    for style in text_styles(envelope) {
        layout::evidence(style.layout, &mut found);
        if style.outline.is_some() {
            found.insert("text.style.stroke".to_owned());
        }
        if style.background.is_some() {
            found.insert("text.style.background".to_owned());
        }
        if style.spans.len() >= 2 {
            found.insert("text.layout.rich-spans".to_owned());
        }
        if style.path.is_some() {
            found.insert("text.layout.path".to_owned());
        }
        if let Some(animation) = &style.animation {
            animation_evidence(animation, &mut found);
        }
    }
    color_evidence(envelope, &mut found);
    found
}

fn positioned(visual: &veac_ir::VisualProperties) -> bool {
    let placement = match visual.placement {
        Placement::Anchor { anchor, inset } => {
            anchor != Anchor::Center || inset.x != 0.0 || inset.y != 0.0
        }
        Placement::Absolute { .. } => true,
    };
    placement
        || !matches!(
            &visual.transform.position,
            Animatable::Constant { value }
                if value.x.value == 0.0 && value.y.value == 0.0
        )
}

fn animation_evidence(animation: &veac_ir::TextAnimation, found: &mut BTreeSet<String>) {
    let unit = match animation.granularity {
        TextGranularity::Whole => "text.animation.unit-whole",
        TextGranularity::Line => "text.animation.unit-line",
        TextGranularity::Word => "text.animation.unit-word",
        TextGranularity::Grapheme => "text.animation.unit-grapheme",
    };
    found.insert(unit.to_owned());
    if matches!(animation.reveal, Animatable::Keyframes { .. })
        && animation.granularity == veac_ir::TextGranularity::Grapheme
    {
        found.insert("text.animation.typewriter".to_owned());
    }
    if matches!(animation.opacity, Animatable::Keyframes { .. }) {
        found.insert("text.animation.fade".to_owned());
    }
    if matches!(
        animation.transform.position_offset,
        Animatable::Keyframes { .. }
    ) {
        found.insert("text.animation.slide".to_owned());
    }
    if matches!(animation.transform.scale, Animatable::Keyframes { .. }) {
        found.insert("text.animation.scale".to_owned());
    }
    if animation
        .highlight
        .as_ref()
        .is_some_and(|highlight| matches!(highlight.progress, Animatable::Keyframes { .. }))
    {
        found.insert("text.animation.highlight-progress".to_owned());
    }
}

fn color_evidence(envelope: &ProjectEnvelope, found: &mut BTreeSet<String>) {
    let mut has_pipeline = false;
    for visual in crate::support::clips(envelope).filter_map(|clip| clip.visual.as_ref()) {
        let Some(pipeline) = &visual.color_pipeline else {
            continue;
        };
        has_pipeline = true;
        for stage in &pipeline.stages {
            match stage {
                ColorStage::Basic { .. } => {
                    found.insert("color.adjust".to_owned());
                    found.insert("color.stage.basic".to_owned());
                }
                ColorStage::Matrix { .. } => {
                    found.insert("color.adjust".to_owned());
                    found.insert("color.stage.matrix".to_owned());
                }
                ColorStage::Hsl { .. } => color_stage(found, "hsl"),
                ColorStage::Curves { .. } => color_stage(found, "curves"),
                ColorStage::Wheels { .. } => color_stage(found, "wheels"),
                ColorStage::Lut { application } => {
                    found.insert("color.stage.lut".to_owned());
                    let kind = envelope
                        .project
                        .materials
                        .iter()
                        .find(|material| material.id == application.material_id)
                        .map(|material| material.kind);
                    match kind {
                        Some(MaterialKind::Lut1d) => {
                            found.insert("color.lut-1d".to_owned());
                        }
                        Some(MaterialKind::Lut3d) => {
                            found.insert("color.lut-3d".to_owned());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    if has_pipeline && has_translucent_generator(envelope) {
        found.insert("color.alpha".to_owned());
    }
}

fn color_stage(found: &mut BTreeSet<String>, id: &str) {
    found.insert("color.adjust".to_owned());
    found.insert(format!("color.{id}"));
    found.insert(format!("color.stage.{id}"));
}

fn has_translucent_generator(envelope: &ProjectEnvelope) -> bool {
    envelope.project.sequences.iter().any(|sequence| {
        sequence
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .any(|clip| {
                matches!(&clip.source,
                    ClipSource::Generated {
                        generator: Generator::Gradient {
                            gradient: Gradient::Linear { stops, .. },
                        },
                    } if stops.iter().any(|stop| stop.color.alpha < 255))
            })
    })
}
