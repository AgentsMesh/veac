use veac_plan::canonical::{FontStyle, TextOverflow, TextWrap, TextWritingMode};
use veac_plan::ResolvedTextStyle;

use super::super::failure::Failure;

pub(super) fn style(value: &ResolvedTextStyle) -> Result<(), Failure> {
    supported(value)?;
    numbers(value)
}

fn supported(value: &ResolvedTextStyle) -> Result<(), Failure> {
    let unsupported = if !value.fallback_fonts.is_empty() {
        Some(("CAPTION_ASS_FALLBACK_FONTS", "font fallback stacks"))
    } else if value.font_style == FontStyle::Oblique {
        Some(("CAPTION_ASS_OBLIQUE", "oblique font style"))
    } else if value.line_height != 1.0 {
        Some(("CAPTION_ASS_LINE_HEIGHT", "authored line height"))
    } else if value.layout.box_width_pixels.is_some()
        || value.layout.box_height_pixels.is_some()
        || value.layout.wrap != TextWrap::None
        || value.layout.overflow != TextOverflow::Visible
    {
        Some(("CAPTION_ASS_TEXT_BOX", "text box wrapping or overflow"))
    } else if value.layout.writing_mode != TextWritingMode::HorizontalTb {
        Some(("CAPTION_ASS_WRITING_MODE", "vertical writing mode"))
    } else if value.path.is_some() {
        Some(("CAPTION_ASS_TEXT_PATH", "text paths"))
    } else if value.background.is_some() {
        Some(("CAPTION_ASS_BACKGROUND", "padded text backgrounds"))
    } else if value.animation.is_some() {
        Some(("CAPTION_ASS_ANIMATION", "text animation"))
    } else {
        None
    };
    match unsupported {
        Some((code, feature)) => Err(Failure::unsupported(
            code,
            format!("ASS sidecars cannot preserve {feature}"),
        )),
        None => Ok(()),
    }
}

fn numbers(value: &ResolvedTextStyle) -> Result<(), Failure> {
    let valid = value.size_pixels.is_finite()
        && value.size_pixels > 0.0
        && value.tracking_pixels.is_finite()
        && value
            .outline
            .as_ref()
            .is_none_or(|item| item.width_pixels.is_finite() && item.width_pixels >= 0.0)
        && value.shadow.as_ref().is_none_or(|item| {
            item.blur_pixels.is_finite()
                && item.blur_pixels >= 0.0
                && item.opacity.is_finite()
                && (0.0..=1.0).contains(&item.opacity)
                && item.offset.x.is_finite()
                && item.offset.y.is_finite()
        });
    if valid {
        Ok(())
    } else {
        Err(Failure::invalid(
            "CAPTION_ASS_STYLE_INVALID",
            "caption style contains invalid numeric values",
        ))
    }
}
