use unicode_segmentation::UnicodeSegmentation;
use veac_plan::canonical::TextGranularity;

use super::model::{AnimatedLine, AnimatedPiece, GlyphStyle, RenderLine};

pub(super) fn annotate(
    lines: &[RenderLine],
    granularity: TextGranularity,
) -> (Vec<AnimatedLine>, usize) {
    match granularity {
        TextGranularity::Whole => map_lines(lines, |_, _, _| 0, 1),
        TextGranularity::Line => map_lines(lines, |line, _, _| line, lines.len().max(1)),
        TextGranularity::Grapheme => graphemes(lines),
        TextGranularity::Word => words(lines),
    }
}

fn graphemes(lines: &[RenderLine]) -> (Vec<AnimatedLine>, usize) {
    let mut unit = 0;
    let mut output = Vec::with_capacity(lines.len());
    for line in lines {
        let mut pieces = Vec::new();
        for piece in &line.pieces {
            for grapheme in piece.text.graphemes(true) {
                push(&mut pieces, grapheme, &piece.style, unit);
                unit += 1;
            }
        }
        if pieces.is_empty() {
            push(&mut pieces, "", &line.pieces[0].style, unit);
            unit += 1;
        }
        output.push(copy_line(line, pieces));
    }
    (output, unit.max(1))
}

fn words(lines: &[RenderLine]) -> (Vec<AnimatedLine>, usize) {
    let mut base = 0;
    let mut output = Vec::with_capacity(lines.len());
    for line in lines {
        let text: String = line
            .pieces
            .iter()
            .map(|piece| piece.text.as_str())
            .collect();
        let starts = word_starts(&text);
        let count = starts.len().max(1);
        let mut pieces = Vec::new();
        let mut offset = 0;
        for piece in &line.pieces {
            split_words(&mut pieces, piece, offset, &starts, base);
            offset += piece.text.len();
        }
        if pieces.is_empty() {
            push(&mut pieces, "", &line.pieces[0].style, base);
        }
        output.push(copy_line(line, pieces));
        base += count;
    }
    (output, base.max(1))
}

fn word_starts(text: &str) -> Vec<usize> {
    if !text.chars().any(char::is_whitespace) {
        return text
            .unicode_word_indices()
            .map(|(start, _)| start)
            .collect();
    }
    let mut after_whitespace = true;
    text.char_indices()
        .filter_map(|(start, character)| {
            let whitespace = character.is_whitespace();
            let begins_token = !whitespace && after_whitespace;
            after_whitespace = whitespace;
            begins_token.then_some(start)
        })
        .collect()
}

fn split_words(
    output: &mut Vec<AnimatedPiece>,
    piece: &super::model::LinePiece,
    offset: usize,
    starts: &[usize],
    base: usize,
) {
    let mut cuts = vec![0, piece.text.len()];
    cuts.extend(
        starts
            .iter()
            .filter(|start| **start > offset && **start < offset + piece.text.len())
            .map(|start| start - offset),
    );
    cuts.sort_unstable();
    cuts.dedup();
    for range in cuts.windows(2) {
        let absolute = offset + range[0];
        let word = starts.partition_point(|start| *start <= absolute);
        let unit = base + word.saturating_sub(1);
        push(output, &piece.text[range[0]..range[1]], &piece.style, unit);
    }
}

fn map_lines(
    lines: &[RenderLine],
    unit: impl Fn(usize, usize, &GlyphStyle) -> usize,
    count: usize,
) -> (Vec<AnimatedLine>, usize) {
    let output = lines
        .iter()
        .enumerate()
        .map(|(line_index, line)| {
            let pieces = line
                .pieces
                .iter()
                .enumerate()
                .map(|(piece_index, piece)| AnimatedPiece {
                    text: piece.text.clone(),
                    style: piece.style.clone(),
                    unit: unit(line_index, piece_index, &piece.style),
                })
                .collect();
            copy_line(line, pieces)
        })
        .collect();
    (output, count)
}

fn push(output: &mut Vec<AnimatedPiece>, text: &str, style: &GlyphStyle, unit: usize) {
    if let Some(previous) = output
        .last_mut()
        .filter(|piece| piece.unit == unit && piece.style == *style)
    {
        previous.text.push_str(text);
    } else {
        output.push(AnimatedPiece {
            text: text.to_owned(),
            style: style.clone(),
            unit,
        });
    }
}

fn copy_line(line: &RenderLine, pieces: Vec<AnimatedPiece>) -> AnimatedLine {
    AnimatedLine {
        pieces,
        width: line.width,
        height: line.height,
        x: line.x,
        y: line.y,
    }
}

#[cfg(test)]
mod tests;
