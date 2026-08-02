mod values;

use crate::program::dependency_budget::DependencyBudget;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{self, Value, ValueLookup};
use crate::program::model::{ComponentInterface, InstanceDecl, ParameterDecl, ResolvedComponent};

pub(super) use values::BoundValues;

pub(super) fn validate_bindings(
    caller_path: &str,
    instance: &InstanceDecl,
    interface: &ComponentInterface,
) -> Result<(), Diagnostic> {
    if let Some(name) = instance
        .bindings
        .keys()
        .find(|name| !interface.has_parameter(name))
    {
        return Err(Diagnostic::new(
            "PROGRAM_PARAMETER_UNKNOWN",
            caller_path,
            format!("parameter `{name}` is not declared by the component"),
            instance.bindings[name].span,
        ));
    }
    Ok(())
}

pub(super) fn bind<'component>(
    caller_path: &str,
    caller_values: &dyn ValueLookup,
    instance: &InstanceDecl,
    component: &'component ResolvedComponent,
    limit: usize,
) -> Result<BoundValues<'component>, Diagnostic> {
    let mut resolver = Parameters {
        caller_path,
        caller_values,
        instance,
        component,
        declarations: &component.interface,
        values: BoundValues::new(component.captured.values.as_ref(), &component.interface),
        active: Vec::new(),
        budget: DependencyBudget::default(),
        limit,
    };
    for declaration in &component.declaration.parameters {
        resolver.value(&declaration.name)?;
    }
    Ok(resolver.values)
}

struct Parameters<'caller, 'component> {
    caller_path: &'caller str,
    caller_values: &'caller dyn ValueLookup,
    instance: &'caller InstanceDecl,
    component: &'component ResolvedComponent,
    declarations: &'component ComponentInterface,
    values: BoundValues<'component>,
    active: Vec<String>,
    budget: DependencyBudget,
    limit: usize,
}

impl Parameters<'_, '_> {
    fn value(&mut self, name: &str) -> Result<(), Diagnostic> {
        if self.values.is_resolved(name) {
            return Ok(());
        }
        if let Some(start) = self.active.iter().position(|value| value == name) {
            let mut chain = self.active[start..].to_vec();
            chain.push(name.to_owned());
            return Err(Diagnostic::new(
                "PROGRAM_PARAMETER_CYCLE",
                self.component.source_path.as_str(),
                format!(
                    "component parameter dependency cycle: {}",
                    chain.join(" -> ")
                ),
                declaration(self.component, self.declarations, name).span,
            ));
        }
        let declaration = declaration(self.component, self.declarations, name);
        self.budget.enter(
            &self.component.source_path,
            "component parameter",
            declaration.span,
        )?;
        self.active.push(name.to_owned());
        let result = self.evaluate(declaration);
        self.active.pop();
        self.budget.leave();
        let value = result?;
        self.values
            .insert(&self.component.source_path, name, value, self.limit)
    }

    fn evaluate(&mut self, declaration: &ParameterDecl) -> Result<Value, Diagnostic> {
        let (source, environment, path, span): (&str, &dyn ValueLookup, &str, _) =
            if let Some(binding) = self.instance.bindings.get(&declaration.name) {
                (
                    &binding.source,
                    self.caller_values,
                    self.caller_path,
                    binding.span,
                )
            } else if let Some(default) = &declaration.default {
                let references =
                    expression::referenced_symbols(&default.source).map_err(|error| {
                        self.expression_error(error, &self.component.source_path, default.span)
                    })?;
                for dependency in references {
                    if self.declarations.has_parameter(&dependency) {
                        self.value(&dependency)?;
                    }
                }
                (
                    &default.source,
                    &self.values,
                    self.component.source_path.as_str(),
                    default.span,
                )
            } else {
                return Err(Diagnostic::new(
                    "PROGRAM_PARAMETER_MISSING",
                    self.caller_path,
                    format!("parameter `{}` has no binding or default", declaration.name),
                    self.instance.span,
                ));
            };
        let value = expression::evaluate_with(source, environment)
            .map_err(|error| self.expression_error(error, path, span))?;
        if value.kind() != declaration.value_type {
            return Err(Diagnostic::new(
                "PROGRAM_PARAMETER_TYPE",
                path,
                format!(
                    "parameter `{}` expects {}, got {}",
                    declaration.name,
                    declaration.value_type,
                    value.kind()
                ),
                span,
            ));
        }
        Ok(value)
    }

    fn expression_error(
        &self,
        error: expression::ExpressionError,
        path: &str,
        span: crate::authoring::Span,
    ) -> Diagnostic {
        Diagnostic::new(
            "PROGRAM_PARAMETER_EXPRESSION",
            path,
            error.to_string(),
            span,
        )
    }
}

fn declaration<'a>(
    component: &'a ResolvedComponent,
    interface: &ComponentInterface,
    name: &str,
) -> &'a ParameterDecl {
    let index = interface
        .parameter(name)
        .expect("component parameter index must match its declaration");
    &component.declaration.parameters[index]
}
