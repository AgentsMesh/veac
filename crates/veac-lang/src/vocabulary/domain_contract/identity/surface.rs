use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::super::super::{StandardLibrarySpec, VocabularyValidationError};

#[derive(Clone, Copy)]
pub(super) enum Exposure<'a> {
    FreeFunction(&'a str),
    Method { receiver: u16, name: &'a str },
}

pub(super) fn collect(
    library: &StandardLibrarySpec,
) -> Result<BTreeMap<u16, Exposure<'_>>, VocabularyValidationError> {
    let mut values = BTreeMap::new();
    for value in &library.free_functions {
        insert(
            &mut values,
            value.operation_opcode,
            Exposure::FreeFunction(&value.name),
        )?;
    }
    for value in &library.methods {
        insert(
            &mut values,
            value.operation_opcode,
            Exposure::Method {
                receiver: value.receiver.opcode,
                name: &value.name,
            },
        )?;
    }
    Ok(values)
}

fn insert<'a>(
    values: &mut BTreeMap<u16, Exposure<'a>>,
    opcode: u16,
    exposure: Exposure<'a>,
) -> Result<(), VocabularyValidationError> {
    if values.insert(opcode, exposure).is_some() {
        return Err(super::error(
            "domain operation has more than one standard-library exposure",
        ));
    }
    Ok(())
}

pub(super) fn update(digest: &mut Sha256, value: &Exposure<'_>) {
    match value {
        Exposure::FreeFunction(name) => {
            digest.update([0x00]);
            super::framed(digest, name.as_bytes());
        }
        Exposure::Method { receiver, name } => {
            digest.update([0x01]);
            digest.update(receiver.to_be_bytes());
            super::framed(digest, name.as_bytes());
        }
    }
}
