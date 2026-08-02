use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::program::dependency_budget::DependencyBudget;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{self, Value};
use crate::program::model::{ConstDecl, Scope, SurfaceFile};

use super::retained;

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &mut Scope,
    retained: &mut retained::Budget,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut declarations = BTreeMap::new();
    for declaration in &file.constants {
        if declarations.contains_key(&declaration.name)
            || scope.values.contains_key(&declaration.name)
        {
            return Err(duplicate(file, declaration));
        }
        declarations.insert(declaration.name.clone(), declaration.clone());
    }
    let exported = declarations
        .values()
        .filter(|value| value.exported)
        .map(|value| value.name.clone())
        .collect();
    let names = declarations.keys().cloned().collect::<Vec<_>>();
    let mut resolver = Constants {
        file,
        declarations,
        values: Arc::make_mut(&mut scope.values),
        active: Vec::new(),
        budget: DependencyBudget::default(),
        retained,
    };
    for name in names {
        resolver.value(&name)?;
    }
    Ok(exported)
}

struct Constants<'a> {
    file: &'a SurfaceFile,
    declarations: BTreeMap<String, ConstDecl>,
    values: &'a mut BTreeMap<String, Arc<Value>>,
    active: Vec<String>,
    budget: DependencyBudget,
    retained: &'a mut retained::Budget,
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
        let references = expression::referenced_symbols(&declaration.expression)
            .map_err(|error| expression_error(self.file, declaration, error))?;
        for dependency in references {
            if self.declarations.contains_key(&dependency) {
                self.value(&dependency)?;
            }
        }
        let value = expression::evaluate_with(&declaration.expression, self.values)
            .map_err(|error| expression_error(self.file, declaration, error))?;
        if value.kind() != declaration.value_type {
            return Err(Diagnostic::new(
                "PROGRAM_CONST_TYPE",
                &self.file.path,
                format!(
                    "constant `{}` expects {}, got {}",
                    declaration.name,
                    declaration.value_type,
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
    Diagnostic::new(
        "PROGRAM_CONST_EXPRESSION",
        &file.path,
        error.to_string(),
        declaration.expression_span,
    )
}
