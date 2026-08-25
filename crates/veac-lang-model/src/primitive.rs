#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimitiveType {
    Integer,
    Scalar,
    Time,
    Length,
    Percent,
    Angle,
    Text,
    Color,
    Boolean,
    Identifier,
}

impl PrimitiveType {
    pub const ALL: [Self; 10] = [
        Self::Integer,
        Self::Scalar,
        Self::Time,
        Self::Length,
        Self::Percent,
        Self::Angle,
        Self::Text,
        Self::Color,
        Self::Boolean,
        Self::Identifier,
    ];
    pub const TOKENS: &'static [&'static str] = &[
        "int",
        "scalar",
        "time",
        "length",
        "percent",
        "angle",
        "text",
        "color",
        "bool",
        "identifier",
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Integer => "int",
            Self::Scalar => "scalar",
            Self::Time => "time",
            Self::Length => "length",
            Self::Percent => "percent",
            Self::Angle => "angle",
            Self::Text => "text",
            Self::Color => "color",
            Self::Boolean => "bool",
            Self::Identifier => "identifier",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.as_str() == value)
    }

    pub const fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Scalar | Self::Time | Self::Length | Self::Percent | Self::Angle
        )
    }
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
