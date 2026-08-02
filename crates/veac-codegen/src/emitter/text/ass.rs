use std::fmt::Write;
use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use veac_plan::ResolvedTextStyle;

use super::animation::Sample;
use super::ass_fonts;
use super::escape::{ass_name, filter_escape};
use super::fonts::EmbeddedFont;
use super::model::AssLayout;

mod line_events;
mod placed;

pub(super) fn script(
    surface: (u32, u32),
    style: &ResolvedTextStyle,
    layout: &AssLayout,
    samples: &[Sample],
    fonts: &[EmbeddedFont],
) -> String {
    let mut output = header(surface, layout);
    output.push_str(&ass_fonts::section(fonts));
    output.push_str("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    match layout {
        AssLayout::Lines(lines) => line_events::events(&mut output, lines, samples, style),
        AssLayout::Placed(pieces) => placed::events(&mut output, pieces, samples, style),
    }
    output
}

pub(super) fn filter(script: &str, font_directory: &Path) -> String {
    let data = STANDARD.encode(script.as_bytes());
    format!(
        "subtitles=filename='data\\:application/x-ass;base64\\,{data}':alpha=1:wrap_unicode=0:fontsdir='{}'",
        filter_escape(&font_directory.to_string_lossy())
    )
}

fn header(surface: (u32, u32), layout: &AssLayout) -> String {
    let font = match layout {
        AssLayout::Lines(lines) => lines
            .iter()
            .flat_map(|line| &line.pieces)
            .next()
            .map(|piece| piece.style.font_name.as_str()),
        AssLayout::Placed(pieces) => pieces.first().map(|piece| piece.style.font_name.as_str()),
    }
    .unwrap_or("Arial");
    format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {}\nPlayResY: {}\nScaledBorderAndShadow: yes\nWrapStyle: 2\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,{},48,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,7,0,0,0,1\n\n",
        surface.0,
        surface.1,
        ass_name(font)
    )
}

pub(super) fn event(output: &mut String, layer: u8, sample: &Sample, text: &str) {
    let _ = writeln!(
        output,
        "Dialogue: {layer},{},{},Default,,0,0,0,,{text}",
        timestamp(sample.start),
        timestamp(sample.end)
    );
}

fn timestamp(seconds: f64) -> String {
    let centiseconds = (seconds.max(0.0) * 100.0).round() as u64;
    format!(
        "{}:{:02}:{:02}.{:02}",
        centiseconds / 360_000,
        centiseconds / 6_000 % 60,
        centiseconds / 100 % 60,
        centiseconds % 100
    )
}
