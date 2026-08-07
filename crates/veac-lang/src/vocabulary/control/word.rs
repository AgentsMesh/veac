#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlWord(&'static str);

impl ControlWord {
    pub(in crate::vocabulary) const fn new(spelling: &'static str) -> Self {
        Self(spelling)
    }

    pub fn all() -> Vec<Self> {
        let mut words = super::public_uses()
            .map(super::ControlUse::word)
            .collect::<Vec<_>>();
        words.sort_unstable();
        words.dedup();
        words
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    pub fn parse(value: &str) -> Option<Self> {
        super::public_uses()
            .map(super::ControlUse::word)
            .find(|word| word.as_str() == value)
    }
}
