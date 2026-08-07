pub(crate) struct DecodedEscape {
    pub character: char,
    pub bytes: usize,
}

pub(crate) fn decode_escape(source: &str) -> Result<DecodedEscape, String> {
    let first = source
        .chars()
        .next()
        .ok_or_else(|| "escape sequence is incomplete".to_owned())?;
    let simple = match first {
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        '"' => Some('"'),
        '\\' => Some('\\'),
        _ => None,
    };
    if let Some(character) = simple {
        return Ok(DecodedEscape {
            character,
            bytes: first.len_utf8(),
        });
    }
    if first != 'u' || !source[first.len_utf8()..].starts_with('{') {
        return Err(format!("unsupported escape `\\{first}`"));
    }
    let digits_start = first.len_utf8() + 1;
    let close = source[digits_start..]
        .find('}')
        .map(|offset| digits_start + offset)
        .ok_or_else(|| "Unicode escape is missing `}`".to_owned())?;
    let digits = &source[digits_start..close];
    if digits.is_empty()
        || digits.len() > 6
        || !digits.bytes().all(|value| value.is_ascii_hexdigit())
    {
        return Err("Unicode escape must contain 1 to 6 hex digits".to_owned());
    }
    let character = u32::from_str_radix(digits, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(|| "Unicode escape is not a scalar value".to_owned())?;
    Ok(DecodedEscape {
        character,
        bytes: close + 1,
    })
}

pub(crate) fn quote(value: &str) -> String {
    let mut rendered = String::with_capacity(value.len() + 2);
    rendered.push('"');
    for character in value.chars() {
        match character {
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            '"' => rendered.push_str("\\\""),
            '\\' => rendered.push_str("\\\\"),
            value if value.is_control() => {
                rendered.push_str(&format!("\\u{{{:x}}}", u32::from(value)))
            }
            value => rendered.push(value),
        }
    }
    rendered.push('"');
    rendered
}

#[cfg(test)]
#[path = "string_codec/tests.rs"]
mod tests;
