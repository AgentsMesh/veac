use veac_ir::{EditOperation, MulticamGroupId, MulticamSync};

use crate::ProviderOutput;

pub(super) fn matches(
    source: &ProviderOutput,
    operation: &EditOperation,
    group_id: &MulticamGroupId,
    indices: &[u32],
) -> bool {
    let ProviderOutput::MulticamSync(result) = source else {
        return false;
    };
    let EditOperation::SetMulticamGroup {
        group_id: operation_group,
        sync,
        angles,
    } = operation
    else {
        return false;
    };
    operation_group == group_id
        && complete_indices(indices, result.offsets.len())
        && sync
            == &MulticamSync {
                basis: result.basis,
                reference_angle_id: result.reference_angle_id.clone(),
            }
        && angles.len() == result.offsets.len()
        && angles.iter().zip(&result.offsets).all(|(angle, offset)| {
            angle.id == offset.angle_id
                && angle.material_id == offset.material_id
                && equivalent_time(angle.source_offset, offset.source_offset)
        })
}

fn complete_indices(values: &[u32], len: usize) -> bool {
    values.len() == len
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| usize::try_from(*value).ok() == Some(index))
}

fn equivalent_time(actual: veac_ir::RationalTime, expected: veac_ir::RationalTime) -> bool {
    i128::from(actual.value) * i128::from(expected.timescale)
        == i128::from(expected.value) * i128::from(actual.timescale)
}
