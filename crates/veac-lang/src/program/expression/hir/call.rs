use super::TypedNode;

#[derive(Debug, Clone)]
pub(in crate::program::expression) struct TypedCallArgument {
    pub slot: usize,
    pub value: TypedNode,
}

#[derive(Debug, Clone)]
pub(in crate::program::expression) struct TypedDefaultArgument {
    pub slot: usize,
    pub target: crate::program::expression::FunctionId,
    pub value_type: crate::program::expression::ValueType,
}
