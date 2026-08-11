use std::sync::Arc;

use crate::authoring::Span;

use super::super::diagnostic::Diagnostic;
use super::super::model::Scope;
use super::retained;
use crate::program::MethodRegistryBuilder;

mod type_names;

pub(super) fn namespace(
    path: &str,
    alias: &str,
    imported: &Scope,
    target: &mut Scope,
    span: Span,
    retained: &mut retained::Budget,
) -> Result<(), Diagnostic> {
    type_names::namespace(path, alias, imported, target, span, retained)?;
    let mut methods = MethodRegistryBuilder::new();
    methods
        .merge(&target.methods, &target.types)
        .and_then(|_| methods.merge(&imported.methods, &target.types))
        .map_err(|error| Diagnostic::new(error.code(), path, error.message(), span))?;
    target.methods = Arc::new(methods.finish());
    let functions = Arc::make_mut(&mut target.functions);
    functions.register_namespace(alias);
    functions.merge_registry_from(&imported.functions);
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
    for (name, _) in imported.functions.iter() {
        let qualified = format!("{alias}.{name}");
        ensure_available(
            path,
            "imported function",
            &qualified,
            target.functions.lookup(&qualified).is_some(),
            span,
        )?;
        retained.alias(path, &qualified, span)?;
        Arc::make_mut(&mut target.functions).bind_from(qualified, &imported.functions, name);
    }
    Ok(())
}

pub(super) fn prelude_types(
    path: &str,
    imported: &Scope,
    target: &mut Scope,
    span: Span,
    retained: &mut retained::Budget,
) -> Result<(), Diagnostic> {
    type_names::prelude(path, imported, target, span, retained)
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
