use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Animatable, EffectInstance};

use super::error::ExecutableLowerError;
use super::{animation, id, value};

mod state;
mod video;

pub(super) struct Context<'a> {
    graph: &'a FrozenDomainGraph,
    timebase: u32,
    scope: &'a [&'a str],
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
    scope: &[&str],
) -> Result<Vec<EffectInstance>, ExecutableLowerError> {
    let context = Context {
        graph,
        timebase,
        scope,
    };
    entity
        .updates(Op::ItemWithEffect)
        .map(|values| {
            let [_, effect] = values else {
                return Err(malformed());
            };
            one(&context, effect)
        })
        .collect()
}

fn one(context: &Context<'_>, source: &Value) -> Result<EffectInstance, ExecutableLowerError> {
    let (operation, values) = value::description(context.graph, source)?;
    let key = value::identifier(values.first())?;
    let effect_state = state::lower(
        context.graph,
        values.get(1).ok_or_else(malformed)?,
        context.timebase,
    )?;
    let effect = video::lower(context, operation, values, key)?;
    let mut path = context.scope.to_vec();
    path.push(key);
    Ok(EffectInstance {
        id: id::effect(&path),
        enabled: effect_state.enabled,
        enable_range: effect_state.range,
        effect,
    })
}

pub(super) fn description(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<EffectInstance, ExecutableLowerError> {
    one(
        &Context {
            graph,
            timebase,
            scope,
        },
        source,
    )
}

pub(super) fn curve_scope<'a>(context: &Context<'a>, key: &'a str, name: &'a str) -> Vec<&'a str> {
    let mut path = Vec::with_capacity(context.scope.len() + 2);
    path.extend_from_slice(context.scope);
    path.push(key);
    path.push(name);
    path
}

pub(super) fn scalar_curve(
    context: &Context<'_>,
    source: &Value,
    scope: &[&str],
) -> Result<Animatable<f64>, ExecutableLowerError> {
    animation::scalar(context.graph, source, context.timebase, scope)
}

pub(super) fn percent_curve(
    context: &Context<'_>,
    source: &Value,
    scope: &[&str],
) -> Result<Animatable<f64>, ExecutableLowerError> {
    animation::percent(context.graph, source, context.timebase, scope)
}

pub(super) fn malformed() -> ExecutableLowerError {
    value::graph("an executable built-in effect has invalid typed topology")
}
