use veac_plan::canonical::MaterialKind;

use super::{at, limit, strip_comment, triple, Failure, Header};

const MAX_1D_SIZE: usize = 65_536;
const MAX_3D_SIZE: usize = 64;

pub(super) fn apply(
    line: &str,
    number: usize,
    table_started: bool,
    expected: MaterialKind,
    header: &mut Header,
) -> Result<bool, Failure> {
    if let Some(value) = arguments(line, "TITLE") {
        require_header(number, table_started)?;
        title(value, number, header)?;
        return Ok(true);
    }
    let content = strip_comment(line).trim();
    let mut fields = content.split_whitespace();
    let Some(keyword) = fields.next() else {
        return Ok(true);
    };
    match keyword {
        "DOMAIN_MIN" => {
            require_header(number, table_started)?;
            set_domain(content, number, true, header)?;
            Ok(true)
        }
        "DOMAIN_MAX" => {
            require_header(number, table_started)?;
            set_domain(content, number, false, header)?;
            Ok(true)
        }
        "LUT_1D_SIZE" => {
            require_header(number, table_started)?;
            set_size(fields, number, MaterialKind::Lut1d, expected, header)?;
            Ok(true)
        }
        "LUT_3D_SIZE" => {
            require_header(number, table_started)?;
            set_size(fields, number, MaterialKind::Lut3d, expected, header)?;
            Ok(true)
        }
        value if value.starts_with(|character: char| character.is_ascii_alphabetic()) => {
            Err(at(number, format!("unknown Cube directive {value}")))
        }
        _ => Ok(false),
    }
}

fn title(value: &str, line: usize, header: &mut Header) -> Result<(), Failure> {
    if header.title_seen {
        return Err(at(line, "TITLE may appear at most once"));
    }
    let Some(body) = value.strip_prefix('"') else {
        return Err(at(line, "TITLE must be a quoted non-empty string"));
    };
    let Some(end) = body.find('"') else {
        return Err(at(line, "TITLE must be a quoted non-empty string"));
    };
    let title = &body[..end];
    let trailing = body[end + 1..].trim();
    if title.is_empty()
        || title.chars().any(char::is_control)
        || !(trailing.is_empty() || trailing.starts_with('#'))
    {
        return Err(at(line, "TITLE must be a quoted non-empty string"));
    }
    header.title_seen = true;
    Ok(())
}

fn set_domain(
    content: &str,
    line: usize,
    minimum: bool,
    header: &mut Header,
) -> Result<(), Failure> {
    let values = content
        .split_once(char::is_whitespace)
        .map_or("", |(_, value)| value);
    let parsed = triple(
        values,
        line,
        if minimum { "DOMAIN_MIN" } else { "DOMAIN_MAX" },
    )?;
    let slot = if minimum {
        &mut header.domain_min
    } else {
        &mut header.domain_max
    };
    if slot.replace(parsed).is_some() {
        return Err(at(line, "Cube domain directives may not be repeated"));
    }
    Ok(())
}

fn set_size<'a>(
    mut fields: impl Iterator<Item = &'a str>,
    line: usize,
    kind: MaterialKind,
    expected: MaterialKind,
    header: &mut Header,
) -> Result<(), Failure> {
    if header.size.is_some() {
        return Err(at(line, "Cube must declare exactly one LUT size"));
    }
    if kind != expected {
        return Err(at(
            line,
            "Cube dimension does not match the LUT material kind",
        ));
    }
    let size = fields
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|_| fields.next().is_none())
        .ok_or_else(|| at(line, "LUT size must be one integer"))?;
    let maximum = if kind == MaterialKind::Lut1d {
        MAX_1D_SIZE
    } else {
        MAX_3D_SIZE
    };
    if size < 2 {
        return Err(at(line, "LUT size must be at least 2"));
    }
    if size > maximum {
        return Err(limit(format!(
            "line {line}: LUT dimension {size} exceeds the {maximum} entry budget"
        )));
    }
    header.size = Some((kind, size));
    Ok(())
}

fn arguments<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    line.strip_prefix(keyword).and_then(|remaining| {
        remaining
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
            .then(|| remaining.trim_start())
    })
}

fn require_header(line: usize, table_started: bool) -> Result<(), Failure> {
    if table_started {
        Err(at(line, "Cube directives must precede table data"))
    } else {
        Ok(())
    }
}
