#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainOpsetVersion(u16);

impl DomainOpsetVersion {
    pub const V1: Self = Self(1);
    pub const V2: Self = Self(2);
    pub const V3: Self = Self(3);
    pub const V4: Self = Self(4);
    pub const V5: Self = Self(5);
    pub const V6: Self = Self(6);
    pub const V7: Self = Self(7);
    pub const CURRENT: Self = Self::V7;

    pub const fn from_raw(value: u16) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for DomainOpsetVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
