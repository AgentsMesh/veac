use std::ops::Range;

use crate::program::expression::{ExpressionError, Value};
use crate::program::{DomainOperationContract, DomainType};

use super::super::error;

pub(super) fn validate(
    contract: &DomainOperationContract,
    values: &[Value],
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    if contract.result().domain_type() != Some(DomainType::ContentIdentity) {
        return Ok(());
    }
    let valid = matches!(values, [Value::Text(value)]
        if value.len() == 64
            && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    valid.then_some(()).ok_or_else(|| {
        error(
            "DOMAIN_CONTENT_IDENTITY",
            "content identity requires exactly 64 lowercase hexadecimal characters",
            span.clone(),
        )
    })
}
