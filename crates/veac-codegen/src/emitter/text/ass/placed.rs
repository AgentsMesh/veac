use veac_plan::ResolvedTextStyle;

use super::event;
use crate::emitter::text::animation::{Sample, UnitSample};
use crate::emitter::text::ass_tags::{
    alpha, color, fill_decoration, glyph, shadow_decoration, shadow_glyph, text,
};
use crate::emitter::text::model::PlacedPiece;
use crate::emitter::time;

pub(super) fn events(
    output: &mut String,
    pieces: &[PlacedPiece],
    samples: &[Sample],
    style: &ResolvedTextStyle,
) {
    for sample in samples {
        for piece in pieces {
            let Some(unit) = sample.units.get(piece.unit) else {
                continue;
            };
            if let Some(background) = &style.background {
                background_event(output, piece, sample, unit, background);
            }
        }
    }
    if let Some(shadow) = &style.shadow {
        for sample in samples {
            for piece in pieces {
                if let Some(unit) = sample.units.get(piece.unit) {
                    shadow_event(output, piece, sample, unit, shadow);
                }
            }
        }
    }
    for sample in samples {
        for piece in pieces {
            if let Some(unit) = sample.units.get(piece.unit) {
                fill_event(output, piece, sample, unit, style);
            }
        }
    }
}

fn fill_event(
    output: &mut String,
    piece: &PlacedPiece,
    sample: &Sample,
    unit: &UnitSample,
    style: &ResolvedTextStyle,
) {
    let (x, y, rotation) = transformed(piece, unit);
    let mut value = format!(
        "{{\\an5\\q2\\pos({},{})\\frz{}\\fscx{}\\fscy{}{}}}",
        time::number(x),
        time::number(y),
        time::number(rotation),
        time::number(unit.scale.0 * 100.0),
        time::number(unit.scale.1 * 100.0),
        fill_decoration(style),
    );
    value.push_str(&glyph(
        &piece.style,
        unit.opacity,
        unit.fill_override,
        style,
    ));
    value.push_str(&text(&piece.text));
    event(output, 2, sample, &value);
}

fn shadow_event(
    output: &mut String,
    piece: &PlacedPiece,
    sample: &Sample,
    unit: &UnitSample,
    shadow: &veac_plan::canonical::Shadow,
) {
    let (x, y, rotation) = transformed(piece, unit);
    let mut value = format!(
        "{{\\an5\\q2\\pos({},{})\\frz{}\\fscx{}\\fscy{}{}}}",
        time::number(x + shadow.offset.x),
        time::number(y + shadow.offset.y),
        time::number(rotation),
        time::number(unit.scale.0 * 100.0),
        time::number(unit.scale.1 * 100.0),
        shadow_decoration(shadow),
    );
    value.push_str(&shadow_glyph(&piece.style, unit.opacity, shadow));
    value.push_str(&text(&piece.text));
    event(output, 1, sample, &value);
}

fn background_event(
    output: &mut String,
    piece: &PlacedPiece,
    sample: &Sample,
    unit: &UnitSample,
    background: &veac_plan::canonical::TextBackground,
) {
    if unit.opacity <= 0.0 {
        return;
    }
    let (x, y, rotation) = transformed(piece, unit);
    let half_width = piece.width / 2.0 + background.padding_pixels;
    let half_height = piece.height / 2.0 + background.padding_pixels;
    let value = format!(
        "{{\\an5\\pos({},{})\\frz{}\\fscx{}\\fscy{}\\p1\\bord0\\shad0\\1c{}\\1a{}}}m {} {} l {} {} {} {} {} {}",
        time::number(x),
        time::number(y),
        time::number(rotation),
        time::number(unit.scale.0 * 100.0),
        time::number(unit.scale.1 * 100.0),
        color(background.color),
        alpha(background.color, unit.opacity),
        time::number(-half_width),
        time::number(-half_height),
        time::number(half_width),
        time::number(-half_height),
        time::number(half_width),
        time::number(half_height),
        time::number(-half_width),
        time::number(half_height),
    );
    event(output, 0, sample, &value);
}

fn transformed(piece: &PlacedPiece, unit: &UnitSample) -> (f64, f64, f64) {
    let radians = unit.rotation_degrees.to_radians();
    let dx = (piece.x - piece.anchor_x) * unit.scale.0;
    let dy = (piece.y - piece.anchor_y) * unit.scale.1;
    let x = piece.anchor_x + dx * radians.cos() - dy * radians.sin() + unit.offset.0;
    let y = piece.anchor_y + dx * radians.sin() + dy * radians.cos() + unit.offset.1;
    (x, y, piece.rotation_degrees + unit.rotation_degrees)
}
