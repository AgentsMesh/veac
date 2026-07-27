use veac_plan::canonical::{FontRef, MaterialId, TextOverflow, TextWrap, TextWritingMode};
use veac_plan::{
    ResolvedClipSource, ResolvedFont, ResolvedInputKind, ResolvedRenderPlan, ResolvedText,
};

use super::{visual, Check};

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        let content = match &clip.source {
            ResolvedClipSource::Text { content } | ResolvedClipSource::Caption { content, .. } => {
                content
            }
            _ => continue,
        };
        if let Some((code, message)) = specific_contract_error(content) {
            check.push(code, Some(clip.id.to_string()), message);
            continue;
        }
        if !style_valid(plan, content) {
            check.push(
                "PLAN_TEXT_INVALID",
                Some(clip.id.to_string()),
                "resolved text style, font, span, layout, or path is invalid",
            );
        }
    }
}

fn specific_contract_error(content: &ResolvedText) -> Option<(&'static str, &'static str)> {
    let style = &content.style;
    if let Some(path) = &style.path {
        if !(2..=256).contains(&path.points.len()) {
            return Some((
                "TEXT_PATH_POINT_LIMIT",
                "text path requires between 2 and 256 points",
            ));
        }
        if style.layout.writing_mode != TextWritingMode::HorizontalTb {
            return Some((
                "TEXT_PATH_WRITING_MODE",
                "text path requires horizontal writing mode",
            ));
        }
    }
    if style.layout.writing_mode != TextWritingMode::HorizontalTb
        && (style.layout.wrap != TextWrap::None || style.layout.overflow == TextOverflow::Ellipsis)
    {
        return Some((
            "TEXT_VERTICAL_LAYOUT_INVALID",
            "vertical text requires no wrapping and no ellipsis overflow",
        ));
    }
    None
}

fn style_valid(plan: &ResolvedRenderPlan, content: &ResolvedText) -> bool {
    let style = &content.style;
    !content.text.is_empty()
        && content.text.len() <= veac_plan::canonical::MAX_TEXT_BYTES
        && content.text.chars().count() <= veac_plan::canonical::MAX_TEXT_SCALARS
        && style.fallback_fonts.len() <= veac_plan::canonical::MAX_FALLBACK_FONTS
        && style.spans.len() <= veac_plan::canonical::MAX_TEXT_SPANS
        && style.size_pixels.is_finite()
        && (0.0..=veac_plan::canonical::MAX_TEXT_SIZE_PIXELS).contains(&style.size_pixels)
        && style.tracking_pixels.is_finite()
        && style.tracking_pixels.abs() <= veac_plan::canonical::MAX_TEXT_TRACKING_PIXELS
        && style.line_height.is_finite()
        && (0.0..=veac_plan::canonical::MAX_TEXT_LINE_HEIGHT).contains(&style.line_height)
        && font_valid(plan, &style.font)
        && style
            .fallback_fonts
            .iter()
            .all(|font| font_valid(plan, font))
        && layout_valid(style.layout)
        && style.path.as_ref().is_none_or(|path| {
            (2..=256).contains(&path.points.len())
                && path.points.windows(2).all(|pair| pair[0] != pair[1])
                && path.start_offset.value.is_finite()
                && path
                    .points
                    .iter()
                    .all(|point| point.x.value.is_finite() && point.y.value.is_finite())
                && style.layout.writing_mode == TextWritingMode::HorizontalTb
                && style.layout.wrap == TextWrap::None
        })
        && style.background.as_ref().is_none_or(|value| {
            value.padding_pixels.is_finite()
                && (0.0..=veac_plan::canonical::MAX_TEXT_PADDING_PIXELS)
                    .contains(&value.padding_pixels)
        })
        && style.outline.as_ref().is_none_or(|value| {
            value.width_pixels.is_finite()
                && (0.0..=veac_plan::canonical::MAX_TEXT_OUTLINE_PIXELS)
                    .contains(&value.width_pixels)
        })
        && style.shadow.as_ref().is_none_or(visual::shadow)
        && spans_valid(plan, content)
}

fn layout_valid(value: veac_plan::canonical::TextLayout) -> bool {
    veac_plan::canonical::text_box_valid(value.box_width_pixels, value.box_height_pixels)
        && (value.wrap == TextWrap::None || value.box_width_pixels.is_some())
        && (value.overflow == TextOverflow::Visible
            || (value.box_width_pixels.is_some() && value.box_height_pixels.is_some()))
        && (value.writing_mode == TextWritingMode::HorizontalTb
            || (value.wrap == TextWrap::None && value.overflow != TextOverflow::Ellipsis))
}

fn spans_valid(plan: &ResolvedRenderPlan, content: &ResolvedText) -> bool {
    let length = content.text.chars().count() as u32;
    let mut previous_end = 0;
    content.style.spans.iter().all(|span| {
        let valid = span.start < span.end
            && span.start >= previous_end
            && span.end <= length
            && span.font.as_ref().is_none_or(|font| font_valid(plan, font))
            && span.size_pixels.is_none_or(|value| {
                value.is_finite()
                    && (0.0..=veac_plan::canonical::MAX_TEXT_SIZE_PIXELS).contains(&value)
            });
        previous_end = span.end;
        valid
    })
}

fn font_valid(plan: &ResolvedRenderPlan, value: &ResolvedFont) -> bool {
    let FontRef::Material { material_id } = &value.requested else {
        return false;
    };
    if MaterialId::new(material_id.as_str()).is_err()
        || value
            .family
            .as_ref()
            .is_some_and(|item| item.trim().is_empty())
        || value
            .postscript_name
            .as_ref()
            .is_some_and(|item| item.trim().is_empty())
    {
        return false;
    }
    let mut inputs = plan
        .inputs
        .iter()
        .filter(|input| input.id == value.input_id);
    let Some(input) = inputs.next() else {
        return false;
    };
    inputs.next().is_none()
        && input.material_id.as_ref() == Some(material_id)
        && matches!(
            &input.kind,
            ResolvedInputKind::Font {
                family,
                postscript_name,
                face_index,
            } if family == &value.family
                && postscript_name == &value.postscript_name
                && *face_index == value.face_index
        )
}
