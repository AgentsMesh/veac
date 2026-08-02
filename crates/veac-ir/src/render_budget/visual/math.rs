use super::super::arithmetic::ceil_div;

pub(super) fn ceil_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let bits = 128 - value.leading_zeros();
    let mut low = 1_u128;
    let mut high = 1_u128 << bits.div_ceil(2);
    while low < high {
        let middle = low + (high - low) / 2;
        if middle >= ceil_div(value, middle) {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}
