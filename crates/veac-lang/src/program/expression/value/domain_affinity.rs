use super::domain::DomainGraphScope;
use super::{Value, ValueConstructionError};

pub(super) fn validate<'a>(
    values: impl IntoIterator<Item = &'a Value>,
) -> Result<(), ValueConstructionError> {
    let mut graph = None;
    for value in values {
        merge(&mut graph, affinity(value)?)?;
    }
    Ok(())
}

pub(super) fn affinity(value: &Value) -> Result<Option<DomainGraphScope>, ValueConstructionError> {
    match value {
        Value::Domain(value) => Ok(Some(value.graph().clone())),
        Value::List(value) => sequence(value.values()),
        Value::Tuple(value) => sequence(value.values()),
        Value::Struct(value) => sequence(value.fields()),
        Value::Enum(value) => sequence(value.fields()),
        Value::Closure(value) => sequence(value.captures()),
        Value::Map(value) => {
            let mut graph = None;
            for entry in value.entries() {
                merge(&mut graph, affinity(entry.key())?)?;
                merge(&mut graph, affinity(entry.value())?)?;
            }
            Ok(graph)
        }
        _ => Ok(None),
    }
}

fn sequence(values: &[Value]) -> Result<Option<DomainGraphScope>, ValueConstructionError> {
    let mut graph = None;
    for value in values {
        merge(&mut graph, affinity(value)?)?;
    }
    Ok(graph)
}

fn merge(
    target: &mut Option<DomainGraphScope>,
    candidate: Option<DomainGraphScope>,
) -> Result<(), ValueConstructionError> {
    let Some(candidate) = candidate else {
        return Ok(());
    };
    match target {
        Some(current) if current != &candidate => Err(ValueConstructionError::new(
            "VALUE_DOMAIN_CROSS_GRAPH",
            "one value cannot retain handles from different graph transactions",
        )),
        Some(_) => Ok(()),
        None => {
            *target = Some(candidate);
            Ok(())
        }
    }
}
