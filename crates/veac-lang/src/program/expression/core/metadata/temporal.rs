use super::CoreValueMetadata;

impl CoreValueMetadata {
    pub(crate) fn temporal_compose<'a>(operands: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut output = Self::constant();
        for operand in operands {
            output.absorb_leaf(operand);
        }
        output
    }

    pub(crate) fn temporal_project(value: &Self) -> Self {
        Self::temporal_compose([value])
    }
}
