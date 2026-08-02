use std::collections::BTreeMap;
use std::sync::Arc;

use crate::authoring::Span;

use super::super::diagnostic::Diagnostic;
use super::super::model::Scope;
use super::retained;

pub(super) fn insert<K: Ord, V>(
    path: &str,
    kind: &str,
    display_name: &str,
    target: &mut BTreeMap<K, V>,
    key: K,
    value: V,
    span: Span,
) -> Result<(), Diagnostic> {
    if target.insert(key, value).is_some() {
        return Err(Diagnostic::new(
            "PROGRAM_DUPLICATE_SYMBOL",
            path,
            format!("{kind} `{display_name}` is declared more than once"),
            span,
        ));
    }
    Ok(())
}

pub(super) fn namespace(
    path: &str,
    alias: &str,
    imported: &Scope,
    target: &mut Scope,
    span: Span,
    retained: &mut retained::Budget,
) -> Result<(), Diagnostic> {
    for (name, value) in imported.values.iter() {
        let qualified = format!("{alias}.{name}");
        ensure_available(
            path,
            "imported constant",
            &qualified,
            target.values.contains_key(&qualified),
            span,
        )?;
        retained.alias(path, &qualified, span)?;
        Arc::make_mut(&mut target.values).insert(qualified, Arc::clone(value));
    }
    for ((name, kind), value) in imported.presets.iter() {
        let qualified = format!("{alias}.{name}");
        let key = (qualified.clone(), *kind);
        ensure_available(
            path,
            "imported preset",
            &qualified,
            target.presets.contains_key(&key),
            span,
        )?;
        retained.alias(path, &qualified, span)?;
        Arc::make_mut(&mut target.presets).insert(key, Arc::clone(value));
    }
    for (name, key) in &imported.components {
        let qualified = format!("{alias}.{name}");
        ensure_available(
            path,
            "imported component",
            &qualified,
            target.components.contains_key(&qualified),
            span,
        )?;
        retained.alias(path, &qualified, span)?;
        insert(
            path,
            "imported component",
            &qualified,
            &mut target.components,
            qualified.clone(),
            key.clone(),
            span,
        )?;
    }
    Ok(())
}

fn ensure_available(
    path: &str,
    kind: &str,
    display_name: &str,
    occupied: bool,
    span: Span,
) -> Result<(), Diagnostic> {
    if occupied {
        Err(Diagnostic::new(
            "PROGRAM_DUPLICATE_SYMBOL",
            path,
            format!("{kind} `{display_name}` is declared more than once"),
            span,
        ))
    } else {
        Ok(())
    }
}
