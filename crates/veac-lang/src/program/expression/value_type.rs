pub use veac_lang_model::{
    FunctionEffect, MapKeyType, ValueType, ValueTypeError, ValueTypeKind, ValueTypeParseError,
    MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH,
};

pub(crate) fn retained_shape_bytes(value: &ValueType) -> Option<usize> {
    use std::mem::size_of;

    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Some(0),
        ValueTypeKind::Nominal(value) => Some(value.diagnostic_name().len()),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => {
            size_of::<ValueType>().checked_add(retained_shape_bytes(value)?)
        }
        ValueTypeKind::Tuple(values) => retained_sequence_bytes(values),
        ValueTypeKind::Function {
            parameters, result, ..
        } => retained_sequence_bytes(parameters)?
            .checked_add(size_of::<ValueType>())?
            .checked_add(retained_shape_bytes(result)?),
    }
}

pub(crate) fn retained_sequence_bytes(values: &[ValueType]) -> Option<usize> {
    use std::mem::size_of;

    values.iter().try_fold(
        values.len().checked_mul(size_of::<ValueType>())?,
        |bytes, value| bytes.checked_add(retained_shape_bytes(value)?),
    )
}

crate::impl_syntax_tokens!(
    FunctionEffect,
    FunctionEffect::Pure => "pure",
    FunctionEffect::Local => "local",
    FunctionEffect::Emit => "emit",
    FunctionEffect::Any => "any",
);

crate::define_syntax_tokens! {
    array
    pub(crate) enum TypeConstructor {
        List => "list",
        Map => "map",
        Range => "range",
        Function => "fn",
    }
}
