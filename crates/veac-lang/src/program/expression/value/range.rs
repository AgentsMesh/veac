use super::ValueConstructionError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeValue {
    start: i64,
    end: i64,
    step: i64,
    count: u64,
}

impl RangeValue {
    pub fn new(start: i64, end: i64, step: i64) -> Result<Self, ValueConstructionError> {
        if step == 0 {
            return Err(ValueConstructionError::new(
                "VALUE_RANGE_STEP",
                "range step cannot be zero",
            ));
        }
        let count = count(start, end, step).ok_or_else(|| {
            ValueConstructionError::new("VALUE_RANGE_COUNT", "range count overflows u64")
        })?;
        Ok(Self {
            start,
            end,
            step,
            count,
        })
    }

    pub fn start(&self) -> i64 {
        self.start
    }

    pub fn end(&self) -> i64 {
        self.end
    }

    pub fn step(&self) -> i64 {
        self.step
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn render(&self) -> String {
        if self.step == 1 {
            format!("{} .. {}", self.start, self.end)
        } else {
            format!("{} .. {} by {}", self.start, self.end, self.step)
        }
    }
}

fn count(start: i64, end: i64, step: i64) -> Option<u64> {
    let forward = step > 0;
    if (forward && start >= end) || (!forward && start <= end) {
        return Some(0);
    }
    let distance = if forward {
        i128::from(end).checked_sub(i128::from(start))?
    } else {
        i128::from(start).checked_sub(i128::from(end))?
    };
    let stride = i128::from(step).checked_abs()?;
    let count = distance
        .checked_sub(1)?
        .checked_div(stride)?
        .checked_add(1)?;
    u64::try_from(count).ok()
}
