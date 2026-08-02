use veac_ir::{Rational, RationalTime};

use crate::{TemplateError, TemplateErrorKind};

pub(crate) fn convert_exact(
    time: RationalTime,
    timescale: u32,
) -> Result<RationalTime, TemplateError> {
    if !time.is_valid() || timescale == 0 {
        return Err(inexact("source duration is invalid"));
    }
    let numerator = i128::from(time.value) * i128::from(timescale);
    let divisor = i128::from(time.timescale);
    if numerator % divisor != 0 {
        return Err(inexact(
            "source duration is not exact at the project timebase",
        ));
    }
    let value = i64::try_from(numerator / divisor)
        .ok()
        .filter(|value| value.unsigned_abs() <= veac_ir::MAX_SAFE_INTEGER)
        .ok_or_else(|| inexact("source duration exceeds the exact time domain"))?;
    Ok(RationalTime { value, timescale })
}

pub(crate) fn ratio(numerator: i64, denominator: i64) -> Result<Rational, TemplateError> {
    if numerator <= 0 || denominator <= 0 {
        return Err(inexact("template fill rate must be positive"));
    }
    let divisor = gcd(numerator.unsigned_abs(), denominator.unsigned_abs());
    let top = numerator / i64::try_from(divisor).expect("positive gcd fits i64");
    let bottom = denominator / i64::try_from(divisor).expect("positive gcd fits i64");
    let bottom = u32::try_from(bottom)
        .map_err(|_| inexact("template fill rate denominator is not representable"))?;
    Ok(Rational {
        numerator: top,
        denominator: bottom,
    })
}

pub(crate) fn centered_start(
    natural: RationalTime,
    needed: RationalTime,
) -> Result<RationalTime, TemplateError> {
    let difference = natural.value - needed.value;
    if difference % 2 != 0 {
        return Err(inexact(
            "center fill start is not exact at the project timebase",
        ));
    }
    Ok(RationalTime {
        value: difference / 2,
        timescale: needed.timescale,
    })
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

fn inexact(message: &str) -> TemplateError {
    TemplateError::new(TemplateErrorKind::InexactTime, message)
}
