#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSequencePattern<'a> {
    pub prefix: &'a str,
    pub suffix: &'a str,
    pub minimum_width: Option<usize>,
}

impl<'a> ImageSequencePattern<'a> {
    pub fn parse(value: &'a str) -> Option<Self> {
        const MAX_FILE_NAME_BYTES: usize = 255;
        const MAX_U64_DIGITS: usize = 20;
        if value.len() > MAX_FILE_NAME_BYTES
            || value
                .chars()
                .any(|character| character.is_control() || matches!(character, '/' | '\\'))
        {
            return None;
        }
        let marker = value.find('%')?;
        let prefix = &value[..marker];
        let tail = &value[marker + 1..];
        let (consumed, minimum_width) = if let Some(suffix) = tail.strip_prefix('d') {
            (tail.len() - suffix.len(), None)
        } else {
            let digits = tail.strip_prefix('0')?;
            let width_len = digits.bytes().take_while(u8::is_ascii_digit).count();
            let width = digits.get(..width_len)?;
            if width.starts_with('0') || digits.as_bytes().get(width_len) != Some(&b'd') {
                return None;
            }
            (width_len + 2, Some(width.parse().ok()?))
        };
        let suffix = &tail[consumed..];
        let generated_len = prefix
            .len()
            .checked_add(suffix.len())?
            .checked_add(minimum_width.unwrap_or(1).max(MAX_U64_DIGITS))?;
        (!suffix.contains('%') && generated_len <= MAX_FILE_NAME_BYTES).then_some(Self {
            prefix,
            suffix,
            minimum_width,
        })
    }

    pub fn matches(self, value: &str) -> bool {
        value
            .strip_prefix(self.prefix)
            .and_then(|value| value.strip_suffix(self.suffix))
            .is_some_and(|digits| {
                !digits.is_empty()
                    && digits.bytes().all(|byte| byte.is_ascii_digit())
                    && self.minimum_width.is_none_or(|width| digits.len() >= width)
            })
    }

    pub fn format_index(self, index: u64) -> String {
        let digits = match self.minimum_width {
            Some(width) => format!("{index:0width$}"),
            None => index.to_string(),
        };
        format!("{}{digits}{}", self.prefix, self.suffix)
    }

    pub fn overlaps(self, other: Self) -> bool {
        (0..=255).any(|length| {
            self.accepts_length(length)
                && other.accepts_length(length)
                && (0..length).all(|index| {
                    compatible(
                        self.fixed_byte(length, index),
                        other.fixed_byte(length, index),
                    )
                })
        })
    }

    fn accepts_length(self, length: usize) -> bool {
        length
            .checked_sub(self.prefix.len() + self.suffix.len())
            .is_some_and(|digits| digits >= self.minimum_width.unwrap_or(1))
    }

    fn fixed_byte(self, length: usize, index: usize) -> Option<u8> {
        if index < self.prefix.len() {
            Some(self.prefix.as_bytes()[index])
        } else if index >= length - self.suffix.len() {
            Some(self.suffix.as_bytes()[index - (length - self.suffix.len())])
        } else {
            None
        }
    }
}

fn compatible(left: Option<u8>, right: Option<u8>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        (Some(value), None) | (None, Some(value)) => value.is_ascii_digit(),
        (None, None) => true,
    }
}
