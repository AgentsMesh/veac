#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    Pure,
    LocalMutation,
    GraphEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FunctionEffect {
    Pure,
    Local,
    Emit,
    Any,
}

impl FunctionEffect {
    pub const ALL: [Self; 4] = [Self::Pure, Self::Local, Self::Emit, Self::Any];
    pub const TOKENS: &'static [&'static str] = &["pure", "local", "emit", "any"];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pure => "pure",
            Self::Local => "local",
            Self::Emit => "emit",
            Self::Any => "any",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pure" => Some(Self::Pure),
            "local" => Some(Self::Local),
            "emit" => Some(Self::Emit),
            "any" => Some(Self::Any),
            _ => None,
        }
    }
}

impl std::fmt::Display for FunctionEffect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
