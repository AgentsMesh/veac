use std::mem::{size_of, size_of_val};

use super::DomainOperationContract;

const MAP_ENTRY_BYTES: usize = 64;

pub(super) fn contract(value: &DomainOperationContract) -> Option<usize> {
    let mut bytes = size_of::<DomainOperationContract>()
        .checked_add(value.name().len())?
        .checked_add(value.exposure().name().len())?
        .checked_add(MAP_ENTRY_BYTES * 3)?
        .checked_add(value.name().len())?
        .checked_add(value.exposure().name().len())?;
    for operand in value.operands() {
        bytes = bytes
            .checked_add(size_of_val(operand))?
            .checked_add(operand.name().len())?;
    }
    Some(bytes)
}
