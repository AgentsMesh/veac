use std::fmt::Write;

use super::fonts::EmbeddedFont;

pub(super) fn section(fonts: &[EmbeddedFont]) -> String {
    if fonts.is_empty() {
        return String::new();
    }
    let mut output = String::from("[Fonts]\n");
    for font in fonts {
        let _ = writeln!(output, "fontname: {}", font.file_name);
        let encoded = encode(&font.bytes);
        for line in encoded.as_bytes().chunks(80) {
            output.push_str(std::str::from_utf8(line).expect("font encoding is ASCII"));
            output.push('\n');
        }
    }
    output.push('\n');
    output
}

fn encode(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        output.push(character(first >> 2));
        output.push(character(
            ((first & 0x03) << 4) | (chunk.get(1).copied().unwrap_or(0) >> 4),
        ));
        if let Some(second) = chunk.get(1) {
            output.push(character(
                ((second & 0x0f) << 2) | (chunk.get(2).copied().unwrap_or(0) >> 6),
            ));
        }
        if let Some(third) = chunk.get(2) {
            output.push(character(third & 0x3f));
        }
    }
    output
}

fn character(value: u8) -> char {
    char::from(value + 33)
}

#[cfg(test)]
mod tests;
