mod header;

use veac_plan::canonical::MaterialKind;

pub(super) const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

pub(super) struct Failure {
    pub limit: bool,
    pub message: String,
}

#[derive(Default)]
pub(super) struct Header {
    pub title_seen: bool,
    pub domain_min: Option<[f64; 3]>,
    pub domain_max: Option<[f64; 3]>,
    pub size: Option<(MaterialKind, usize)>,
}

pub(super) fn validate(bytes: &[u8], expected: MaterialKind) -> Result<(), Failure> {
    let source = std::str::from_utf8(bytes).map_err(|_| invalid("LUT must be valid UTF-8"))?;
    let mut header = Header::default();
    let mut rows = 0_usize;
    let mut table_started = false;
    for (index, raw) in source.lines().enumerate() {
        let number = index + 1;
        let raw = if index == 0 {
            raw.strip_prefix('\u{feff}').unwrap_or(raw)
        } else {
            raw
        };
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if header::apply(line, number, table_started, expected, &mut header)? {
            continue;
        }
        let data = strip_comment(line).trim();
        if data.is_empty() {
            continue;
        }
        let Some((_, size)) = header.size else {
            return Err(at(number, "LUT size must be declared before table data"));
        };
        table_started = true;
        triple(data, number, "table row")?;
        let expected_rows = rows_for(expected, size);
        if rows >= expected_rows {
            return Err(at(number, "LUT table contains more rows than declared"));
        }
        rows += 1;
    }
    let Some((_, size)) = header.size else {
        return Err(invalid("LUT must declare exactly one dimension and size"));
    };
    validate_domain(&header)?;
    let expected_rows = rows_for(expected, size);
    if rows != expected_rows {
        return Err(invalid(format!(
            "LUT table has {rows} rows; its declared size requires {expected_rows}"
        )));
    }
    Ok(())
}

fn validate_domain(header: &Header) -> Result<(), Failure> {
    let minimum = header.domain_min.unwrap_or([0.0; 3]);
    let maximum = header.domain_max.unwrap_or([1.0; 3]);
    if minimum
        .into_iter()
        .zip(maximum)
        .all(|(minimum, maximum)| minimum < maximum)
    {
        Ok(())
    } else {
        Err(invalid(
            "DOMAIN_MIN must be strictly below DOMAIN_MAX in every channel",
        ))
    }
}

fn rows_for(kind: MaterialKind, size: usize) -> usize {
    match kind {
        MaterialKind::Lut1d => size,
        MaterialKind::Lut3d => size * size * size,
        _ => 0,
    }
}

pub(super) fn triple(value: &str, line: usize, label: &str) -> Result<[f64; 3], Failure> {
    let mut fields = value.split_whitespace();
    let mut next = || {
        fields
            .next()
            .and_then(|field| field.parse::<f64>().ok())
            .filter(|number| number.is_finite())
            .ok_or_else(|| at(line, format!("{label} must contain three finite numbers")))
    };
    let values = [next()?, next()?, next()?];
    if fields.next().is_some() {
        return Err(at(
            line,
            format!("{label} must contain exactly three numbers"),
        ));
    }
    Ok(values)
}

pub(super) fn strip_comment(value: &str) -> &str {
    value.split_once('#').map_or(value, |(content, _)| content)
}

pub(super) fn at(line: usize, message: impl Into<String>) -> Failure {
    invalid(format!("line {line}: {}", message.into()))
}

pub(super) fn invalid(message: impl Into<String>) -> Failure {
    Failure {
        limit: false,
        message: message.into(),
    }
}

pub(super) fn limit(message: impl Into<String>) -> Failure {
    Failure {
        limit: true,
        message: message.into(),
    }
}
