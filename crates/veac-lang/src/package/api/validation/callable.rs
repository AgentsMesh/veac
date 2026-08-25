use super::{contract, named_types, value_type};
use crate::package::{
    api::{ApiCallableSemantics, ApiFunctionParameter, ApiType},
    PackageError,
};

pub(super) fn validate(
    parameters: &[ApiFunctionParameter],
    return_type: &ApiType,
    semantics: &ApiCallableSemantics,
    has_receiver: bool,
) -> Result<(), PackageError> {
    let mut found_default = false;
    for parameter in parameters {
        if parameter.has_default {
            found_default = true;
        } else if found_default {
            return contract("required API parameters cannot follow a parameter with a default");
        }
    }
    named_types(
        parameters
            .iter()
            .map(|parameter| (&parameter.name, &parameter.value_type)),
    )?;
    value_type(return_type, 0)?;
    if semantics.result.parameters.len() != parameters.len() {
        return contract("callable result dependency count must match parameter arity");
    }
    if semantics.result.receiver.is_some() != has_receiver {
        return contract("callable receiver dependency must match callable kind");
    }
    Ok(())
}
