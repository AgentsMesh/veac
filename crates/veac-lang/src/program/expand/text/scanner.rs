use std::ops::Range;

use crate::program::diagnostic::Diagnostic;

use super::super::budget;

pub(super) fn expression_spans(path: &str, source: &str) -> Result<Vec<Range<usize>>, Diagnostic> {
    let mut spans = Vec::new();
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    let mut state = State::Code;
    while cursor < bytes.len() {
        match state {
            State::Code if starts(bytes, cursor, b"//") => {
                step(&mut state, State::LineComment, &mut cursor)
            }
            State::Code if starts(bytes, cursor, b"/*") => {
                step(&mut state, State::BlockComment, &mut cursor)
            }
            State::Code if bytes[cursor] == b'"' => enter(&mut state, State::String, &mut cursor),
            State::Code if starts(bytes, cursor, b"${") => {
                let end = expression_end(path, source, cursor + 2)?;
                budget::ensure_replacement_available(path, spans.len())?;
                spans.push(cursor..end + 1);
                cursor = end + 1;
            }
            State::String if bytes[cursor] == b'"' && !escaped(bytes, cursor) => {
                enter(&mut state, State::Code, &mut cursor)
            }
            State::LineComment if bytes[cursor] == b'\n' => {
                enter(&mut state, State::Code, &mut cursor)
            }
            State::BlockComment if starts(bytes, cursor, b"*/") => {
                step(&mut state, State::Code, &mut cursor)
            }
            _ => cursor += 1,
        }
    }
    Ok(spans)
}

fn expression_end(path: &str, source: &str, start: usize) -> Result<usize, Diagnostic> {
    let bytes = source.as_bytes();
    let mut cursor = start;
    let mut parentheses = 0usize;
    let mut state = State::Code;
    while cursor < bytes.len() {
        match state {
            State::Code if starts(bytes, cursor, b"//") => {
                step(&mut state, State::LineComment, &mut cursor)
            }
            State::Code if starts(bytes, cursor, b"/*") => {
                step(&mut state, State::BlockComment, &mut cursor)
            }
            State::Code if bytes[cursor] == b'"' => enter(&mut state, State::String, &mut cursor),
            State::Code if bytes[cursor] == b'(' => {
                parentheses += 1;
                cursor += 1;
            }
            State::Code if bytes[cursor] == b')' && parentheses > 0 => {
                parentheses -= 1;
                cursor += 1;
            }
            State::Code if bytes[cursor] == b'}' && parentheses == 0 => return Ok(cursor),
            State::String if bytes[cursor] == b'"' && !escaped(bytes, cursor) => {
                enter(&mut state, State::Code, &mut cursor)
            }
            State::LineComment if bytes[cursor] == b'\n' => {
                enter(&mut state, State::Code, &mut cursor)
            }
            State::BlockComment if starts(bytes, cursor, b"*/") => {
                step(&mut state, State::Code, &mut cursor)
            }
            _ => cursor += 1,
        }
    }
    Err(Diagnostic::new(
        "PROGRAM_EXPRESSION_CLOSE",
        path,
        "expression placeholder is not closed",
        crate::authoring::Span::default(),
    ))
}

fn step(state: &mut State, next: State, cursor: &mut usize) {
    *state = next;
    *cursor += 2;
}

fn enter(state: &mut State, next: State, cursor: &mut usize) {
    *state = next;
    *cursor += 1;
}

fn starts(bytes: &[u8], at: usize, value: &[u8]) -> bool {
    bytes.get(at..at.saturating_add(value.len())) == Some(value)
}

fn escaped(bytes: &[u8], at: usize) -> bool {
    let mut slashes = 0usize;
    let mut cursor = at;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        slashes += 1;
        cursor -= 1;
    }
    slashes % 2 == 1
}

#[derive(Clone, Copy)]
enum State {
    Code,
    String,
    LineComment,
    BlockComment,
}
