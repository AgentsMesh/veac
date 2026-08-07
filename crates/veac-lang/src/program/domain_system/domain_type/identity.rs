use super::classification::DomainTypeClassification;
use super::generated::{COUNT, TABLES};
use super::DomainType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DomainTypeIdentity {
    domain_type: DomainType,
    name: &'static str,
    classification: DomainTypeClassification,
}

impl DomainTypeIdentity {
    pub(super) const fn new(
        domain_type: DomainType,
        name: &'static str,
        classification: DomainTypeClassification,
    ) -> Self {
        Self {
            domain_type,
            name,
            classification,
        }
    }

    pub(super) const fn domain_type(&self) -> DomainType {
        self.domain_type
    }

    pub(super) const fn name(&self) -> &'static str {
        self.name
    }

    pub(super) const fn classification(&self) -> DomainTypeClassification {
        self.classification
    }
}

macro_rules! declare_domain_types {
    ($($constant:ident = $opcode:literal => $name:literal, $classification:ident;)+) => {
        #[allow(non_upper_case_globals)]
        impl DomainType {
            $(pub const $constant: Self = Self::new($opcode);)+
        }

        pub(super) const IDENTITIES: &[DomainTypeIdentity] = &[
            $(DomainTypeIdentity::new(
                DomainType::$constant,
                $name,
                DomainTypeClassification::$classification,
            ),)+
        ];
    };
}

pub(super) use declare_domain_types;

const EMPTY: DomainTypeIdentity = DomainTypeIdentity::new(
    DomainType::new(0),
    "",
    DomainTypeClassification::TopologyValue,
);
const IDENTITIES: [DomainTypeIdentity; COUNT] = ordered_identities();

pub(super) const fn find(opcode: u16) -> Option<DomainTypeIdentity> {
    let mut left = 0;
    let mut right = IDENTITIES.len();
    while left < right {
        let middle = left + (right - left) / 2;
        let current = IDENTITIES[middle].domain_type().opcode();
        if current < opcode {
            left = middle + 1;
        } else {
            right = middle;
        }
    }
    if left < IDENTITIES.len() && IDENTITIES[left].domain_type().opcode() == opcode {
        Some(IDENTITIES[left])
    } else {
        None
    }
}

pub(super) fn find_name(name: &str) -> Option<&'static DomainTypeIdentity> {
    IDENTITIES.iter().find(|identity| identity.name() == name)
}

pub(super) fn all() -> std::slice::Iter<'static, DomainTypeIdentity> {
    IDENTITIES.iter()
}

const fn ordered_identities() -> [DomainTypeIdentity; COUNT] {
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
        while cursor > 0
            && output[cursor].domain_type().opcode() < output[cursor - 1].domain_type().opcode()
        {
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
