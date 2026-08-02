use ethnum::I256;

use super::ExactNumber;

pub(super) fn add(left: ExactNumber, right: ExactNumber) -> Option<ExactNumber> {
    combine(left, right, false)
}

pub(super) fn subtract(left: ExactNumber, right: ExactNumber) -> Option<ExactNumber> {
    combine(left, right, true)
}

fn combine(left: ExactNumber, right: ExactNumber, subtract: bool) -> Option<ExactNumber> {
    let left_numerator = I256::from(left.numerator());
    let left_denominator = I256::from(left.denominator());
    let right_numerator = I256::from(right.numerator());
    let right_denominator = I256::from(right.denominator());
    let left_scaled = left_numerator * right_denominator;
    let right_scaled = right_numerator * left_denominator;
    let numerator = if subtract {
        left_scaled - right_scaled
    } else {
        left_scaled + right_scaled
    };
    narrow(numerator, left_denominator * right_denominator)
}

fn narrow(mut numerator: I256, mut denominator: I256) -> Option<ExactNumber> {
    if numerator == I256::ZERO {
        return Some(ExactNumber::integer(0));
    }
    let divisor = gcd(numerator.abs(), denominator);
    numerator /= divisor;
    denominator /= divisor;
    ExactNumber::new(
        i128::try_from(numerator).ok()?,
        i128::try_from(denominator).ok()?,
    )
}

fn gcd(mut left: I256, mut right: I256) -> I256 {
    while right != I256::ZERO {
        (left, right) = (right, left % right);
    }
    left
}
