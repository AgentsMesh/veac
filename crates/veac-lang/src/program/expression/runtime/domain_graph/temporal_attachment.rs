use std::ops::Range;
use std::sync::Arc;

use super::state::TemporalAttachment;
use super::{accounting, error, validation, DomainGraphTransaction};
use crate::program::expression::{
    ClosureValue, DomainOrigin, ExpressionError, PrimitiveType, TemporalAttachmentKind, Value,
};

impl DomainGraphTransaction<'_> {
    pub(in crate::program::expression) fn attach_temporal(
        &mut self,
        kind: TemporalAttachmentKind,
        owner: Value,
        selectors: Vec<Value>,
        animation: Arc<ClosureValue>,
        span: Range<usize>,
        instance: Option<DomainOrigin>,
    ) -> Result<Value, ExpressionError> {
        self.ensure_active(&span)?;
        let result =
            self.attach_temporal_inner(kind, owner, selectors, animation, span.clone(), instance);
        if result.is_err() {
            self.taint();
        }
        result
    }

    fn attach_temporal_inner(
        &mut self,
        kind: TemporalAttachmentKind,
        owner: Value,
        selectors: Vec<Value>,
        animation: Arc<ClosureValue>,
        span: Range<usize>,
        instance: Option<DomainOrigin>,
    ) -> Result<Value, ExpressionError> {
        let owner_node = validation::node(
            self,
            &owner,
            kind.owner_type().as_domain().expect("closed owner type"),
            &span,
        )?;
        validation::unowned(self, owner_node, &span)?;
        validate_selectors(kind, &selectors, &span)?;
        if animation.value_type() != &kind.animation_type() {
            return Err(error(
                "DOMAIN_TEMPORAL_ANIMATION_TYPE",
                "temporal attachment received an incompatible verified closure",
                span,
            ));
        }
        if self.state.temporal.iter().any(|value| {
            value.owner == owner_node && value.kind == kind && value.selectors.as_ref() == selectors
        }) {
            return Err(error(
                "DOMAIN_TEMPORAL_DUPLICATE",
                "one temporal sink accepts exactly one animation producer",
                span,
            ));
        }
        let animation_value = Value::Closure(Arc::clone(&animation));
        let bytes = accounting::temporal_attachment_bytes(&owner, &selectors, &animation_value)
            .ok_or_else(|| {
                error(
                    "DOMAIN_LOGICAL_SIZE",
                    "logical graph size overflow",
                    span.clone(),
                )
            })?;
        self.execution.reserve_domain_graph(0, bytes, span)?;
        self.state.logical_bytes += bytes;
        self.state.temporal.push(TemporalAttachment {
            owner: owner_node,
            kind,
            selectors: selectors.into(),
            animation,
            instance,
        });
        Ok(owner)
    }
}

fn validate_selectors(
    kind: TemporalAttachmentKind,
    selectors: &[Value],
    span: &Range<usize>,
) -> Result<(), ExpressionError> {
    if selectors.len() != kind.selector_types().len() {
        return Err(error(
            "DOMAIN_TEMPORAL_SELECTOR",
            "temporal selector arity mismatch",
            span.clone(),
        ));
    }
    for (value, expected) in selectors.iter().zip(kind.selector_types()) {
        if value.primitive_kind() != Some(*expected) {
            return Err(error(
                "DOMAIN_TEMPORAL_SELECTOR",
                "temporal selector type mismatch",
                span.clone(),
            ));
        }
        if *expected == PrimitiveType::Integer
            && !matches!(value, Value::Integer(index) if u32::try_from(*index).is_ok())
        {
            return Err(error(
                "DOMAIN_TEMPORAL_SELECTOR",
                "mask ordinal must fit u32",
                span.clone(),
            ));
        }
    }
    Ok(())
}
