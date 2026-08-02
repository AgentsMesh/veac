use veac_plan::canonical::HorizontalTextAlignment;
use veac_plan::ResolvedTextStyle;

use super::super::animation::Sample;
use super::super::ass_tags::{
    alpha, color, fill_decoration, piece as piece_tags, shadow_decoration, shadow_piece,
    text as escaped_text,
};
use super::super::model::AnimatedLine;
use super::event;
use crate::emitter::time;

pub(super) fn events(
    output: &mut String,
    lines: &[AnimatedLine],
    samples: &[Sample],
    style: &ResolvedTextStyle,
) {
    for sample in samples {
        for line in lines {
            if let Some(background) = &style.background {
                background_event(output, line, sample, background);
            }
        }
    }
    if let Some(shadow) = &style.shadow {
        for sample in samples {
            for line in lines {
                shadow_event(output, line, sample, shadow);
            }
        }
    }
    for sample in samples {
        for line in lines {
            fill_event(output, line, sample, style);
        }
    }
}

fn fill_event(
    output: &mut String,
    line: &AnimatedLine,
    sample: &Sample,
    style: &ResolvedTextStyle,
) {
    let (alignment, x) = anchor(line);
    let mut text = format!(
        "{{\\an{alignment}\\q2\\pos({},{}){}}}",
        time::number(x),
        time::number(line.y),
        fill_decoration(style)
    );
    for piece in &line.pieces {
        let unit = sample.units.get(piece.unit);
        let opacity = unit.map_or(0.0, |value| value.opacity);
        let fill = unit.and_then(|value| value.fill_override);
        text.push_str(&piece_tags(piece, opacity, fill, style));
        text.push_str(&escaped_text(&piece.text));
    }
    event(output, 2, sample, &text);
}

fn shadow_event(
    output: &mut String,
    line: &AnimatedLine,
    sample: &Sample,
    shadow: &veac_plan::canonical::Shadow,
) {
    let (alignment, x) = anchor(line);
    let mut text = format!(
        "{{\\an{alignment}\\q2\\pos({},{}){}}}",
        time::number(x + shadow.offset.x),
        time::number(line.y + shadow.offset.y),
        shadow_decoration(shadow)
    );
    for piece in &line.pieces {
        let opacity = sample
            .units
            .get(piece.unit)
            .map_or(0.0, |unit| unit.opacity);
        text.push_str(&shadow_piece(piece, opacity, shadow));
        text.push_str(&escaped_text(&piece.text));
    }
    event(output, 1, sample, &text);
}

fn background_event(
    output: &mut String,
    line: &AnimatedLine,
    sample: &Sample,
    background: &veac_plan::canonical::TextBackground,
) {
    let opacity = line
        .pieces
        .iter()
        .filter_map(|piece| sample.units.get(piece.unit))
        .map(|value| value.opacity)
        .fold(0.0, f64::max);
    if opacity <= 0.0 {
        return;
    }
    let padding = background.padding_pixels;
    let (left, top) = (line.x - padding, line.y - padding);
    let (right, bottom) = (
        line.x + line.width + padding,
        line.y + line.height + padding,
    );
    let text = format!(
        "{{\\an7\\pos(0,0)\\p1\\bord0\\shad0\\1c{}\\1a{}}}m {} {} l {} {} {} {} {} {}",
        color(background.color),
        alpha(background.color, opacity),
        time::number(left),
        time::number(top),
        time::number(right),
        time::number(top),
        time::number(right),
        time::number(bottom),
        time::number(left),
        time::number(bottom),
    );
    event(output, 0, sample, &text);
}

fn anchor(line: &AnimatedLine) -> (u8, f64) {
    match line.alignment {
        HorizontalTextAlignment::Left => (7, line.x),
        HorizontalTextAlignment::Center => (8, line.x + line.width / 2.0),
        HorizontalTextAlignment::Right => (9, line.x + line.width),
    }
}
