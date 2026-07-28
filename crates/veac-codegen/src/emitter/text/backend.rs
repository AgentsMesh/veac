use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{TextGranularity, TextOverflow, TextWrap, TextWritingMode};
use veac_plan::{ResolvedClip, ResolvedText};

use super::animation;
use super::error::TextError;
use super::fonts::FontBook;
use super::model::AssLayout;
use super::{ass, layout, lines, placement, styled, surface, units};
use crate::emitter::canvas::Canvas;

pub(super) struct Backend {
    pub surface: surface::Surface,
    pub script: String,
    pub font_directory: std::path::PathBuf,
    pub font_paths: Vec<std::path::PathBuf>,
}

pub(super) fn build(
    content: &ResolvedText,
    clip: &ResolvedClip,
    bindings: &ExecutionBindings,
    canvas: Canvas,
) -> Result<Backend, TextError> {
    let visual = clip
        .visual
        .as_ref()
        .ok_or_else(|| TextError::invalid("text has no visual properties"))?;
    let sample_count = limits::samples(animation::sample_count(
        content.style.animation.as_ref(),
        clip.record_range.duration,
        canvas.frame_rate,
    ))?;
    let fps = canvas.frame_rate.numerator as f64 / f64::from(canvas.frame_rate.denominator);
    limits::frame_rate(sample_count, fps)?;
    let surface = surface::resolve(canvas, visual, content);
    let dimensions = surface.dimensions();
    validate_layout(content)?;
    let mut fonts = FontBook::load(content, bindings)?;
    let styled = styled::resolve(content, &fonts)?;
    let authored_layout = content.style.layout;
    let geometry_layout = content.style.path.is_some()
        || authored_layout.writing_mode != TextWritingMode::HorizontalTb;
    let box_width = (!geometry_layout)
        .then_some(authored_layout.box_width_pixels)
        .flatten();
    let wrap = if geometry_layout {
        TextWrap::None
    } else {
        authored_layout.wrap
    };
    let shaped = layout::shape(&styled, &mut fonts, box_width, wrap);
    let rendered = lines::compose(
        &styled,
        &shaped,
        content.style.layout,
        dimensions,
        surface.origin,
        &mut fonts,
    )?;
    let granularity = content
        .style
        .animation
        .as_ref()
        .map_or(TextGranularity::Whole, |value| value.granularity);
    let (animated, unit_count) = units::annotate(&rendered, granularity);
    limits::unit_samples(sample_count, unit_count)?;
    let ass_layout = if placement::required(&content.style) {
        AssLayout::Placed(placement::place(
            &animated,
            &content.style,
            &mut fonts,
            dimensions,
            surface.origin,
        )?)
    } else {
        AssLayout::Lines(animated)
    };
    let samples = animation::samples(
        content.style.animation.as_ref(),
        clip.record_range.duration,
        canvas.frame_rate,
        unit_count,
        dimensions,
        sample_count,
    );
    let layers = event_layers(
        content.style.background.is_some(),
        content.style.shadow.is_some(),
    );
    let items = match &ass_layout {
        AssLayout::Lines(lines) => lines.len(),
        AssLayout::Placed(pieces) => pieces.len(),
    };
    let events = samples.len().saturating_mul(items).saturating_mul(layers);
    limits::events(events)?;
    let script = ass::script(
        dimensions,
        &content.style,
        &ass_layout,
        &samples,
        &fonts.embedded,
    );
    limits::script(script.len())?;
    Ok(Backend {
        surface,
        script,
        font_directory: fonts.directory,
        font_paths: fonts.paths,
    })
}

pub(crate) fn event_layers(has_background: bool, has_shadow: bool) -> usize {
    1 + usize::from(has_background) + usize::from(has_shadow)
}

fn validate_layout(content: &ResolvedText) -> Result<(), TextError> {
    let layout = content.style.layout;
    if layout.writing_mode != TextWritingMode::HorizontalTb
        && (layout.wrap != TextWrap::None || layout.overflow == TextOverflow::Ellipsis)
    {
        return Err(TextError::new(
            "TEXT_VERTICAL_LAYOUT_INVALID",
            "vertical text requires wrap none and does not support ellipsis overflow",
        ));
    }
    if content.style.path.is_some() && layout.wrap != TextWrap::None {
        return Err(TextError::new(
            "TEXT_PATH_LAYOUT_INVALID",
            "text path requires wrap none",
        ));
    }
    Ok(())
}

mod limits;
