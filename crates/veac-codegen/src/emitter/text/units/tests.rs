use veac_plan::canonical::{Color, FontStyle, FontWeight, TextGranularity};

use super::super::model::{GlyphStyle, LinePiece, RenderLine};
use super::annotate;

#[test]
fn authored_whitespace_groups_chinese_tokens() {
    let (lines, count) = words(vec![line(&[("按 词语 依次 揭示", 0)])]);

    assert_eq!(count, 4);
    assert_eq!(
        units(&lines[0]),
        vec![("按 ", 0), ("词语 ", 1), ("依次 ", 2), ("揭示", 3)]
    );
}

#[test]
fn authored_whitespace_groups_latin_tokens() {
    let (lines, count) = words(vec![line(&[("alpha beta gamma", 0)])]);

    assert_eq!(count, 3);
    assert_eq!(
        units(&lines[0]),
        vec![("alpha ", 0), ("beta ", 1), ("gamma", 2)]
    );
}

#[test]
fn surrounding_and_repeated_whitespace_add_no_units() {
    let (lines, count) = words(vec![line(&[("  高亮\t进度  ", 0)])]);

    assert_eq!(count, 2);
    assert_eq!(units(&lines[0]), vec![("  高亮\t", 0), ("进度  ", 1)]);
}

#[test]
fn unspaced_chinese_uses_unicode_word_boundaries() {
    let (lines, count) = words(vec![line(&[("连续中文", 0)])]);

    assert_eq!(count, 4);
    assert_eq!(
        units(&lines[0]),
        vec![("连", 0), ("续", 1), ("中", 2), ("文", 3)]
    );
}

#[test]
fn tokens_keep_units_across_rich_style_pieces() {
    let (lines, count) = words(vec![line(&[("词", 0), ("语 ", 1), ("跨", 0), ("样式", 1)])]);

    assert_eq!(count, 2);
    assert_eq!(
        lines[0]
            .pieces
            .iter()
            .map(|piece| (piece.text.as_str(), piece.unit, piece.style.font))
            .collect::<Vec<_>>(),
        vec![("词", 0, 0), ("语 ", 0, 1), ("跨", 1, 0), ("样式", 1, 1)]
    );
}

#[test]
fn word_units_continue_in_reading_order_across_lines() {
    let (lines, count) = words(vec![line(&[("甲 乙", 0)]), line(&[("丙 丁", 0)])]);

    assert_eq!(count, 4);
    assert_eq!(units(&lines[0]), vec![("甲 ", 0), ("乙", 1)]);
    assert_eq!(units(&lines[1]), vec![("丙 ", 2), ("丁", 3)]);
}

fn words(lines: Vec<RenderLine>) -> (Vec<super::super::model::AnimatedLine>, usize) {
    annotate(&lines, TextGranularity::Word)
}

fn line(values: &[(&str, usize)]) -> RenderLine {
    RenderLine {
        pieces: values
            .iter()
            .map(|(text, style)| LinePiece {
                text: (*text).to_owned(),
                style: glyph(*style),
            })
            .collect(),
        width: 0.0,
        height: 0.0,
        x: 0.0,
        y: 0.0,
    }
}

fn glyph(font: usize) -> GlyphStyle {
    GlyphStyle {
        font,
        font_name: format!("font-{font}"),
        font_alias: format!("alias-{font}"),
        weight: FontWeight::Normal,
        font_style: FontStyle::Normal,
        size: 10.0,
        color: Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        },
        tracking: 0.0,
        line_height: 1.0,
    }
}

fn units(line: &super::super::model::AnimatedLine) -> Vec<(&str, usize)> {
    line.pieces
        .iter()
        .map(|piece| (piece.text.as_str(), piece.unit))
        .collect()
}
