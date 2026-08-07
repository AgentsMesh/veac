use super::super::FunctionRegistry;
use crate::program::DomainOperationRegistry;

pub(super) struct VerifyContext<'a> {
    pub(super) functions: &'a FunctionRegistry,
    pub(super) input_trust: &'a dyn Fn(&str) -> bool,
    pub(super) domain: &'a DomainOperationRegistry,
}
