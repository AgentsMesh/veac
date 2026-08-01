mod font;
mod position;
mod style;
mod text;
mod validate;

use std::collections::BTreeSet;
use std::fmt::Write;

use veac_artifact::ExecutionBindings;

use super::{failure::Failure, format::ass_timestamp, Cue};
use crate::emitter::time;
use font::FontCatalog;
use position::Position;
use style::{ass_alpha, ass_rgb, Style};

struct Event {
    layer: usize,
    start: veac_plan::canonical::RationalTime,
    end: veac_plan::canonical::RationalTime,
    style: String,
    speaker: String,
    prefix: String,
    text: String,
}

pub(super) fn render(
    cues: &[Cue<'_>],
    width: u32,
    height: u32,
    bindings: &ExecutionBindings,
) -> Result<String, Failure> {
    let layers = layer_keys(cues);
    let mut fonts = FontCatalog::new(bindings);
    let mut styles = Vec::<(String, Style)>::new();
    let mut events = Vec::with_capacity(cues.len());
    for cue in cues {
        let resolved = cue.content.styled().ok_or_else(|| {
            Failure::invalid(
                "CAPTION_ASS_PRESENTATION_INVALID",
                "ASS sidecar requires styled caption presentation",
            )
            .at(&cue.clip.id)
        })?;
        let font = fonts
            .name(&resolved.font)
            .map_err(|error| error.at(&cue.clip.id))?;
        let style = Style::resolve(resolved, font).map_err(|error| error.at(&cue.clip.id))?;
        let style_name = register(&mut styles, &style);
        let position = position::resolve(cue, resolved, width, height)
            .map_err(|error| error.at(&cue.clip.id))?;
        let speaker = text::speaker(cue.speaker)
            .map_err(|error| error.at(&cue.clip.id))?
            .to_owned();
        let rendered = text::render(cue.content, resolved, &style, &mut fonts)
            .map_err(|error| error.at(&cue.clip.id))?;
        let layer = layers
            .iter()
            .position(|value| value == &cue.layer_key)
            .expect("cue layer key was collected");
        events.push(Event {
            layer,
            start: cue.start,
            end: cue.end,
            style: style_name,
            speaker,
            prefix: prefix(position, &style),
            text: rendered,
        });
    }
    let mut output = header(width, height);
    for (name, style) in &styles {
        style.write_line(&mut output, name);
    }
    output.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    for event in events {
        let _ = writeln!(
            output,
            "Dialogue: {},{},{},{},{},0,0,0,,{{{}}}{}",
            event.layer,
            ass_timestamp(event.start),
            ass_timestamp(event.end),
            event.style,
            event.speaker,
            event.prefix,
            event.text,
        );
    }
    Ok(output)
}

fn register(styles: &mut Vec<(String, Style)>, value: &Style) -> String {
    if let Some((name, _)) = styles.iter().find(|(_, style)| style == value) {
        return name.clone();
    }
    let name = format!("VEAC{:04}", styles.len() + 1);
    styles.push((name.clone(), value.clone()));
    name
}

fn layer_keys(cues: &[Cue<'_>]) -> Vec<(i32, i32, u32, i64, u32, String)> {
    cues.iter()
        .map(|cue| cue.layer_key.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn prefix(position: Position, style: &Style) -> String {
    let mut output = format!(
        "\\an{}\\q2\\pos({},{}){}",
        style.alignment,
        time::number(position.x),
        time::number(position.y),
        style.base_tags(),
    );
    match style.shadow {
        Some((color, opacity, blur, x, y)) => {
            let _ = write!(
                output,
                "\\blur{}\\xshad{}\\yshad{}\\4c{}\\4a{}",
                time::number(blur),
                time::number(x),
                time::number(y),
                ass_rgb(color),
                ass_alpha(color, opacity),
            );
        }
        None => output.push_str("\\shad0"),
    }
    output
}

fn header(width: u32, height: u32) -> String {
    format!(
        "[Script Info]\nScriptType: v4.00+\nWrapStyle: 2\nScaledBorderAndShadow: yes\nPlayResX: {width}\nPlayResY: {height}\n\n\
         [V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n"
    )
}
