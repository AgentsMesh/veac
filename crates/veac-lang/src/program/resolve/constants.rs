use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::program::dependency_budget::DependencyBudget;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{
    self, ExecutionBudget, ExpressionContext, FunctionMap, TypeEnvironment, Value,
};
use crate::program::model::{ConstDecl, Scope, SurfaceFile};

use super::retained;

mod declaration;
mod dependency;

pub(super) use declaration::types as declaration_types;

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &mut Scope,
    retained: &mut retained::Budget,
    execution: &ExecutionBudget,
    functions: Arc<FunctionMap>,
    types: TypeEnvironment,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut declarations = BTreeMap::new();
    for declaration in &file.constants {
        declarations.insert(declaration.name.clone(), declaration.clone());
    }
    let exported = declarations
        .values()
        .filter(|value| value.exported)
        .map(|value| value.name.clone())
        .collect();
    let names = declarations.keys().cloned().collect::<Vec<_>>();
    let context = scope
        .expression_context()
        .with_functions_arc(functions)
        .with_provisional_values(types.clone());
    let mut resolver = Constants {
        file,
        declarations,
        types,
        context,
        values: Arc::make_mut(&mut scope.values),
        active: Vec::new(),
        budget: DependencyBudget::default(),
        retained,
        execution,
    };
    for name in names {
        resolver.value(&name)?;
    }
    Ok(exported)
}

struct Constants<'a> {
    file: &'a SurfaceFile,
    declarations: BTreeMap<String, ConstDecl>,
    types: TypeEnvironment,
    context: ExpressionContext,
    values: &'a mut BTreeMap<String, Arc<Value>>,
    active: Vec<String>,
    budget: DependencyBudget,
    retained: &'a mut retained::Budget,
    execution: &'a ExecutionBudget,
}

impl Constants<'_> {
    fn value(&mut self, name: &str) -> Result<Arc<Value>, Diagnostic> {
        if let Some(value) = self.values.get(name) {
            return Ok(Arc::clone(value));
        }
        if let Some(start) = self.active.iter().position(|value| value == name) {
            let mut chain = self.active[start..].to_vec();
            chain.push(name.to_owned());
            let declaration = &self.declarations[name];
            return Err(Diagnostic::new(
                "PROGRAM_CONST_CYCLE",
                &self.file.path,
                format!("constant dependency cycle: {}", chain.join(" -> ")),
                declaration.expression_span,
            ));
        }
        let declaration = self.declarations[name].clone();
        self.budget
            .enter(&self.file.path, "constant", declaration.expression_span)?;
        self.active.push(name.to_owned());
        let result = self.evaluate(&declaration);
        self.active.pop();
        self.budget.leave();
        let value = Arc::new(result?);
        self.retained
            .value(&self.file.path, name, value.as_ref(), declaration.span)?;
        self.values.insert(name.to_owned(), Arc::clone(&value));
        Ok(value)
    }

    fn evaluate(&mut self, declaration: &ConstDecl) -> Result<Value, Diagnostic> {
        let references = dependency::names(&declaration.expression, &self.types, &self.context)
            .map_err(|error| expression_error(self.file, declaration, error))?;
        for dependency in references {
            if self.declarations.contains_key(&dependency) {
                self.value(&dependency)?;
            }
        }
        let trusted = expression::TrustedValueLookup::new(self.values);
        let value = expression::evaluate_lookup_with_budget(
            &declaration.expression,
            &trusted,
            &self.context,
            self.execution,
        )
        .map_err(|error| expression_error(self.file, declaration, error))?;
        let expected = self
            .types
            .get(&declaration.name)
            .expect("declared constant has a resolved type");
        if *expected != value.value_type() {
            return Err(Diagnostic::new(
                "PROGRAM_CONST_TYPE",
                &self.file.path,
                format!(
                    "constant `{}` expects {}, got {}",
                    declaration.name,
                    expected,
                    value.kind()
                ),
                declaration.expression_span,
            ));
        }
        Ok(value)
    }
}

fn duplicate(file: &SurfaceFile, declaration: &ConstDecl) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_DUPLICATE_SYMBOL",
        &file.path,
        format!("constant `{}` is declared more than once", declaration.name),
        declaration.span,
    )
}

fn expression_error(
    file: &SurfaceFile,
    declaration: &ConstDecl,
    error: expression::ExpressionError,
) -> Diagnostic {
    crate::program::expression_diagnostic::runtime(
        "PROGRAM_CONST_EXPRESSION",
        &file.path,
        declaration.expression_span,
        error,
    )
}
