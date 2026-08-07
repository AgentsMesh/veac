use super::generated::{COUNT, TABLES};
use super::DomainOperationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OperationIdentity {
    id: DomainOperationId,
    name: &'static str,
}

impl OperationIdentity {
    pub(super) const fn new(id: DomainOperationId, name: &'static str) -> Self {
        Self { id, name }
    }

    pub(super) const fn id(&self) -> DomainOperationId {
        self.id
    }

    pub(super) const fn name(&self) -> &'static str {
        self.name
    }
}

macro_rules! declare_operations {
    ($category:ident { $($constant:ident = $opcode:literal => $name:literal;)+ }) => {
        #[allow(non_upper_case_globals)]
        impl DomainOperationId {
            $(pub const $constant: Self = Self::new($opcode);)+
        }

        pub(super) const IDENTITIES: &[OperationIdentity] = &[
            $(OperationIdentity::new(
                DomainOperationId::$constant,
                $name,
            ),)+
        ];
    };
}

pub(super) use declare_operations;

const EMPTY: OperationIdentity = OperationIdentity::new(DomainOperationId::new(0), "");
const IDENTITIES: [OperationIdentity; COUNT] = ordered_identities();

pub(super) const fn find(opcode: u16) -> Option<OperationIdentity> {
    let mut left = 0;
    let mut right = IDENTITIES.len();
    while left < right {
        let middle = left + (right - left) / 2;
        let current = IDENTITIES[middle].id().opcode();
        if current < opcode {
            left = middle + 1;
        } else {
            right = middle;
        }
    }
    if left < IDENTITIES.len() && IDENTITIES[left].id().opcode() == opcode {
        Some(IDENTITIES[left])
    } else {
        None
    }
}

pub(super) fn all() -> std::slice::Iter<'static, OperationIdentity> {
    IDENTITIES.iter()
}

const fn ordered_identities() -> [OperationIdentity; COUNT] {
    let mut output = [EMPTY; COUNT];
    let mut output_index = 0;
    let mut table_index = 0;
    while table_index < TABLES.len() {
        let table = TABLES[table_index];
        let mut identity_index = 0;
        while identity_index < table.len() {
            output[output_index] = table[identity_index];
            output_index += 1;
            identity_index += 1;
        }
        table_index += 1;
    }
    let mut index = 1;
    while index < output.len() {
        let mut cursor = index;
        while cursor > 0 && output[cursor].id().opcode() < output[cursor - 1].id().opcode() {
            let previous = output[cursor - 1];
            output[cursor - 1] = output[cursor];
            output[cursor] = previous;
            cursor -= 1;
        }
        index += 1;
    }
    output
}

#[cfg(test)]
mod tests;
