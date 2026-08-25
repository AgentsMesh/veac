use std::mem::{size_of, size_of_val};

use super::super::{CoreProgram, VerifiedClosureDefinition, VerifiedCoreProgram};

impl VerifiedCoreProgram {
    pub(super) fn retained_bytes(&self) -> Option<usize> {
        let bytes = size_of::<Self>()
            .checked_sub(size_of::<CoreProgram>())?
            .checked_add(self.core().retained_bytes()?)?
            .checked_add(self.nominal_types().retained_bytes())?;
        self.closures().iter().try_fold(
            bytes.checked_add(
                self.closures()
                    .len()
                    .checked_mul(size_of::<std::sync::Arc<VerifiedClosureDefinition>>())?,
            )?,
            |bytes, definition| bytes.checked_add(closure_bytes(definition)?),
        )
    }
}

pub(super) fn closure_bytes(definition: &VerifiedClosureDefinition) -> Option<usize> {
    size_of_val(definition)
        .checked_sub(size_of::<VerifiedCoreProgram>())?
        .checked_add(super::sequence_type_bytes(definition.parameter_types())?)?
        .checked_add(
            definition
                .parameter_stages()
                .len()
                .checked_mul(size_of::<super::super::Stage>())?,
        )?
        .checked_add(super::sequence_type_bytes(definition.capture_types())?)?
        .checked_add(super::value_type_bytes(definition.value_type())?)?
        .checked_add(super::metadata::summary_payload(definition.summary())?)?
        .checked_add(definition.body().retained_bytes()?)
}
