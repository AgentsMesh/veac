use crate::program::expression::ValueType;

pub(super) fn value_type_bytes(value: &ValueType) -> Option<usize> {
    crate::program::expression::value_type::retained_shape_bytes(value)
}

pub(super) fn sequence_type_bytes(values: &[ValueType]) -> Option<usize> {
    crate::program::expression::value_type::retained_sequence_bytes(values)
}
