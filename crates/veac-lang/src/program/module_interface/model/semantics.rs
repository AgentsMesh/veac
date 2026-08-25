use super::super::super::expression::{Effect, Stage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCallableSemantics {
    pub effect: Effect,
    pub contains_local_mutation: bool,
    pub result: ModuleResultSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleResultSemantics {
    pub shape: Stage,
    pub leaf: Stage,
    pub receiver: Option<ModuleParameterDependency>,
    pub parameters: Vec<ModuleParameterDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleParameterDependency {
    pub shape_from_shape: bool,
    pub shape_from_leaf: bool,
    pub leaf_from_shape: bool,
    pub leaf_from_leaf: bool,
}
