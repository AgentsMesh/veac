use super::Definitions;
use crate::program::expression::core::{CoreValueMetadata, ValueId};

pub(super) fn expected(
    owner: ValueId,
    selectors: &[ValueId],
    animation: ValueId,
    definitions: &Definitions,
) -> CoreValueMetadata {
    CoreValueMetadata::temporal_attachment(
        definitions.metadata(owner),
        selectors
            .iter()
            .map(|value| definitions.metadata(*value).clone()),
        definitions.metadata(animation),
    )
}
