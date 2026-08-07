use super::ControlWord;
use crate::vocabulary::{CanonicalRole, GrammarPosition, LanguageLayer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlUse {
    word: ControlWord,
    position: GrammarPosition,
    role: CanonicalRole,
}

impl ControlUse {
    pub(in crate::vocabulary::control) const fn new(
        word: ControlWord,
        position: GrammarPosition,
        role: CanonicalRole,
    ) -> Self {
        Self {
            word,
            position,
            role,
        }
    }

    pub const fn word(self) -> ControlWord {
        self.word
    }

    pub const fn position(self) -> GrammarPosition {
        self.position
    }

    pub const fn layer(self) -> LanguageLayer {
        self.position.layer()
    }

    pub const fn role(self) -> CanonicalRole {
        self.role
    }

    pub const fn as_str(self) -> &'static str {
        self.word.as_str()
    }

    pub fn matches(self, value: &str) -> bool {
        self.as_str() == value
    }
}

#[cfg(test)]
#[path = "use_model/tests.rs"]
mod tests;
