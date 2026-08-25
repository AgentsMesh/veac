mod classification;
mod generated;
mod identity;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainType(u16);

impl DomainType {
    const fn new(opcode: u16) -> Self {
        Self(opcode)
    }

    pub const fn opcode(self) -> u16 {
        self.0
    }

    pub const fn from_opcode(opcode: u16) -> Option<Self> {
        match identity::find(opcode) {
            Some(identity) => Some(identity.domain_type()),
            None => None,
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        identity::find_name(name).map(identity::DomainTypeIdentity::domain_type)
    }

    pub const fn name(self) -> &'static str {
        match identity::find(self.opcode()) {
            Some(identity) => identity.name(),
            None => panic!("invalid closed domain type"),
        }
    }

    pub fn all() -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + Clone {
        identity::all().map(identity::DomainTypeIdentity::domain_type)
    }
}

impl std::fmt::Display for DomainType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

impl std::fmt::Debug for DomainType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

#[cfg(test)]
mod tests;
