use std::cmp::Ordering;

mod wide;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactNumber {
    numerator: i128,
    denominator: i128,
}

impl ExactNumber {
    pub fn new(numerator: i128, denominator: i128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let (numerator, denominator) = if denominator < 0 {
            (numerator.checked_neg()?, denominator.checked_neg()?)
        } else {
            (numerator, denominator)
        };
        if numerator == 0 {
            return Some(Self::integer(0));
        }
        let divisor = gcd(numerator.unsigned_abs(), denominator as u128) as i128;
        Some(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    pub const fn integer(value: i128) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    pub fn numerator(self) -> i128 {
        self.numerator
    }

    pub fn denominator(self) -> i128 {
        self.denominator
    }

    pub fn is_zero(self) -> bool {
        self.numerator == 0
    }

    pub(crate) fn checked_neg(self) -> Option<Self> {
        Self::new(self.numerator.checked_neg()?, self.denominator)
    }

    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        wide::add(self, other)
    }

    pub(crate) fn checked_sub(self, other: Self) -> Option<Self> {
        wide::subtract(self, other)
    }

    pub(crate) fn checked_mul(self, other: Self) -> Option<Self> {
        let first = gcd(self.numerator.unsigned_abs(), other.denominator as u128) as i128;
        let second = gcd(other.numerator.unsigned_abs(), self.denominator as u128) as i128;
        let numerator = (self.numerator / first).checked_mul(other.numerator / second)?;
        let denominator = (self.denominator / second).checked_mul(other.denominator / first)?;
        Self::new(numerator, denominator)
    }

    pub(crate) fn checked_div(self, other: Self) -> Option<Self> {
        if other.is_zero() {
            return None;
        }
        let common_numerator = gcd(
            self.numerator.unsigned_abs(),
            other.numerator.unsigned_abs(),
        );
        let common_denominator = gcd(self.denominator as u128, other.denominator as u128);
        let mut numerator = divide_signed(self.numerator, common_numerator)?
            .checked_mul(other.denominator / common_denominator as i128)?;
        if other.numerator < 0 {
            numerator = numerator.checked_neg()?;
        }
        let denominator_factor = other.numerator.unsigned_abs() / common_numerator;
        let denominator_factor = i128::try_from(denominator_factor).ok()?;
        let denominator =
            (self.denominator / common_denominator as i128).checked_mul(denominator_factor)?;
        Self::new(numerator, denominator)
    }

    pub(crate) fn compare(self, other: Self) -> Ordering {
        match (self.numerator.signum(), other.numerator.signum()) {
            (left, right) if left != right => left.cmp(&right),
            (0, 0) => Ordering::Equal,
            (-1, -1) => compare_unsigned(
                other.numerator.unsigned_abs(),
                other.denominator as u128,
                self.numerator.unsigned_abs(),
                self.denominator as u128,
            ),
            _ => compare_unsigned(
                self.numerator.unsigned_abs(),
                self.denominator as u128,
                other.numerator.unsigned_abs(),
                other.denominator as u128,
            ),
        }
    }

    pub(crate) fn render(self) -> String {
        decimal(self).unwrap_or_else(|| format!("{} / {}", self.numerator, self.denominator))
    }
}

fn divide_signed(value: i128, divisor: u128) -> Option<i128> {
    if divisor == 1_u128 << 127 {
        return (value == i128::MIN).then_some(-1);
    }
    Some(value / i128::try_from(divisor).ok()?)
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

fn compare_unsigned(mut an: u128, mut ad: u128, mut bn: u128, mut bd: u128) -> Ordering {
    let mut reverse = false;
    loop {
        let comparison = (an / ad).cmp(&(bn / bd));
        if comparison != Ordering::Equal {
            return if reverse {
                comparison.reverse()
            } else {
                comparison
            };
        }
        let (ar, br) = (an % ad, bn % bd);
        match (ar == 0, br == 0) {
            (true, true) => return Ordering::Equal,
            (true, false) => {
                return if reverse {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (false, true) => {
                return if reverse {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            _ => (an, ad, bn, bd, reverse) = (ad, ar, bd, br, !reverse),
        }
    }
}

fn decimal(value: ExactNumber) -> Option<String> {
    if value.denominator == 1 {
        return Some(value.numerator.to_string());
    }
    let (twos, rest) = factors(value.denominator, 2);
    let (fives, rest) = factors(rest, 5);
    if rest != 1 {
        return None;
    }
    let places = twos.max(fives);
    let scale = 2_i128
        .checked_pow(places - twos)?
        .checked_mul(5_i128.checked_pow(places - fives)?)?;
    let scaled = value.numerator.checked_mul(scale)?;
    let negative = scaled < 0;
    let mut digits = scaled.unsigned_abs().to_string();
    let places = places as usize;
    if digits.len() <= places {
        digits = format!("{}{}", "0".repeat(places + 1 - digits.len()), digits);
    }
    let split = digits.len() - places;
    let sign = if negative { "-" } else { "" };
    Some(format!("{sign}{}.{}", &digits[..split], &digits[split..]))
}

fn factors(mut value: i128, factor: i128) -> (u32, i128) {
    let mut count = 0;
    while value % factor == 0 {
        value /= factor;
        count += 1;
    }
    (count, value)
}
