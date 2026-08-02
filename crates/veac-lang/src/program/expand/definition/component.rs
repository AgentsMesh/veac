mod fixture;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::Value;
use crate::program::model::{ComponentCatalog, ComponentKey, ResolvedComponent};
use crate::program::parser;

use super::{budget::Budget, validate};

pub(crate) fn component(
    component: &ResolvedComponent,
    catalog: &ComponentCatalog,
    budget: &mut Budget,
) -> Result<(), Diagnostic> {
    for environment in fixture::ENVIRONMENTS {
        validate_in(component, catalog, environment, budget)?;
    }
    Ok(())
}

fn validate_in(
    component: &ResolvedComponent,
    catalog: &ComponentCatalog,
    environment: fixture::Environment,
    budget: &mut Budget,
) -> Result<(), Diagnostic> {
    let source = fixture::source(component, environment);
    let file = parser::parse("<component-validation>", &source)
        .map_err(|errors| definition_error(component, &errors[0].message))?;
    let key = ComponentKey {
        path: component.source_path.clone(),
        name: component.declaration.name.clone(),
    };
    let components = BTreeMap::from([(component.declaration.name.clone(), key)]);
    let mut values = BTreeMap::new();
    values.insert(
        fixture::IDENTIFIER_BINDING.to_owned(),
        Arc::new(Value::Identifier(environment.identifier.to_owned())),
    );
    let presets = BTreeMap::new();
    let caller = super::super::scope::View {
        values: &values,
        presets: &presets,
        components: &components,
    };
    let (expanded, _, component_nodes) = super::super::project_in(&file, caller, catalog)?;
    validate::source(&expanded).map_err(|message| definition_error(component, &message))?;
    // Preserve established semantic diagnostics before charging this bounded validation unit.
    budget.component(
        &component.source_path,
        component.declaration.span,
        component_nodes,
        source.len(),
        expanded.len(),
    )
}

fn definition_error(component: &ResolvedComponent, message: &str) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_COMPONENT_DEFINITION",
        &component.source_path,
        format!(
            "invalid component `{}`: {message}",
            component.declaration.name
        ),
        component.declaration.span,
    )
}
