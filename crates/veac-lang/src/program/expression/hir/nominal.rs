use crate::program::{FieldIndex, TypeId, VariantIndex};

use super::{LocalId, TypedBlock, TypedCallArgument, TypedNode};
use crate::program::expression::core::FunctionId;

#[derive(Debug, Clone)]
pub(crate) struct TypedStructConstruct {
    pub nominal: TypeId,
    pub fields: Vec<(FieldIndex, TypedNode)>,
}

#[derive(Debug, Clone)]
pub(crate) struct TypedEnumConstruct {
    pub nominal: TypeId,
    pub variant: VariantIndex,
    pub fields: Vec<(FieldIndex, TypedNode)>,
}

#[derive(Debug, Clone)]
pub(crate) struct TypedStructProject {
    pub receiver: Box<TypedNode>,
    pub field: FieldIndex,
}

#[derive(Debug, Clone)]
pub(crate) struct TypedPatternBinding {
    pub id: LocalId,
    pub field: FieldIndex,
}

#[derive(Debug, Clone)]
pub(crate) struct TypedMatchArm {
    pub variant: VariantIndex,
    pub bindings: Vec<TypedPatternBinding>,
    pub body: TypedBlock,
}

#[derive(Debug, Clone)]
pub(crate) struct TypedMethodCall {
    pub target: FunctionId,
    pub receiver: Box<TypedNode>,
    pub arguments: Vec<TypedCallArgument>,
    pub defaults: Vec<super::TypedDefaultArgument>,
}
