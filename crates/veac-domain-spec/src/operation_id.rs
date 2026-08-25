mod generated;
mod identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainOperationId(u16);

impl DomainOperationId {
    const fn new(opcode: u16) -> Self {
        Self(opcode)
    }

    pub const fn opcode(self) -> u16 {
        self.0
    }

    pub const fn from_opcode(opcode: u16) -> Option<Self> {
        match identity::find(opcode) {
            Some(identity) => Some(identity.id()),
            None => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match identity::find(self.opcode()) {
            Some(identity) => identity.name(),
            None => panic!("invalid closed domain operation ID"),
        }
    }

    pub fn all() -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator + Clone {
        identity::all().map(identity::OperationIdentity::id)
    }
}

impl std::fmt::Display for DomainOperationId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "0x{:04x}", self.opcode())
    }
}

#[cfg(test)]
#[path = "operation_id/tests.rs"]
mod tests;
