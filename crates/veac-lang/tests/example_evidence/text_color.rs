use std::collections::BTreeSet;

use veac_ir::{
    Anchor, Animatable, ClipSource, ColorStage, MaterialKind, Placement, ProjectEnvelope,
    TextGranularity, TextWritingMode,
};

use crate::support::{assert_preview_evidence, text_styles};

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
    for style in text_styles(envelope) {
        let layout = style.layout;
        if layout.horizontal_alignment != veac_ir::HorizontalTextAlignment::Center
            || layout.vertical_alignment != veac_ir::VerticalTextAlignment::Middle
        {
            found.insert("text.layout.alignment".to_owned());
        }
        match layout.writing_mode {
            TextWritingMode::HorizontalTb => {
                found.insert("text.layout.writing-horizontal".to_owned());
                found.insert("text.layout.writing-horizontal-tb".to_owned());
            }
            TextWritingMode::VerticalRl => {
                found.insert("text.layout.writing-vertical".to_owned());
                found.insert("text.layout.writing-vertical-rl".to_owned());
            }
            TextWritingMode::VerticalLr => {
                found.insert("text.layout.writing-vertical".to_owned());
                found.insert("text.layout.writing-vertical-lr".to_owned());
            }
        }
        if layout.wrap != veac_ir::TextWrap::None {
            found.insert("text.layout.wrap".to_owned());
        }
        if layout.overflow != veac_ir::TextOverflow::Visible {
            found.insert("text.layout.overflow".to_owned());
        }
        if style.outline.is_some() {
            found.insert("text.style.stroke".to_owned());
        }
        if style.background.is_some() {
            found.insert("text.style.background".to_owned());
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
    for visual in crate::support::clips(envelope).filter_map(|clip| clip.visual.as_ref()) {
        let Some(pipeline) = &visual.color_pipeline else {
            continue;
        };
        for stage in &pipeline.stages {
            match stage {
                ColorStage::Basic { .. }
                | ColorStage::Matrix { .. }
                | ColorStage::Hsl { .. }
                | ColorStage::Curves { .. }
                | ColorStage::Wheels { .. } => {
                    found.insert("color.adjust".to_owned());
                }
                ColorStage::Lut { application } => {
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
}
