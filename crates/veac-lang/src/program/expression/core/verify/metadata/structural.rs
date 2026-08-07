use super::super::definitions::Definitions;
use crate::program::expression::core::{CoreValueMetadata, ValueId};

pub(super) fn list(elements: &[ValueId], definitions: &Definitions) -> CoreValueMetadata {
    CoreValueMetadata::list(&values(elements, definitions))
}

pub(super) fn tuple(elements: &[ValueId], definitions: &Definitions) -> CoreValueMetadata {
    CoreValueMetadata::tuple(&values(elements, definitions))
}

pub(super) fn map_key(
    builder: ValueId,
    key: ValueId,
    definitions: &Definitions,
) -> CoreValueMetadata {
    let mut metadata = definitions.metadata(builder).clone();
    metadata.absorb_shape(definitions.metadata(key));
    metadata
}

pub(super) fn map_value(
    pending: ValueId,
    value: ValueId,
    definitions: &Definitions,
) -> CoreValueMetadata {
    let mut metadata = definitions.metadata(pending).clone();
    metadata.append_map_value(definitions.metadata(value));
    metadata
}

fn values(elements: &[ValueId], definitions: &Definitions) -> Vec<CoreValueMetadata> {
    elements
        .iter()
        .map(|id| definitions.metadata(*id).clone())
        .collect()
}
