use std::collections::BTreeSet;
use std::sync::Arc;

use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{
    self, BuiltinFunction, FunctionDefinition, FunctionMap, FunctionOrigin, FunctionParameter,
    TypeEnvironment, MAX_FUNCTION_PARAMETERS,
};
use crate::program::model::{FunctionDecl, Scope, SurfaceFile};

use super::{retained, type_annotations};

pub(super) fn resolve(
    file: &SurfaceFile,
    scope: &mut Scope,
    retained: &mut retained::Budget,
    database: &crate::program::CompilerDatabase,
    admission: &crate::program::compiler_database::DependencyRouteAdmission,
) -> Result<BTreeSet<String>, Diagnostic> {
    validate_unique(file, scope)?;
    let definitions = file
        .functions
        .iter()
        .map(|function| definition(file, scope, function))
        .collect::<Result<Vec<_>, _>>()?;
    let payload_limit = retained.function_payload_limit(&definitions);
    let context = scope.expression_context();
    let compiled = database
        .compile_functions(&context, &definitions, payload_limit)
        .map_err(|error| diagnostic(file, error))?;
    database.consume_invalidation(admission);
    retained.functions(file, compiled.functions(), compiled.methods())?;
    scope.functions = compiled.functions_arc();
    Ok(file
        .functions
        .iter()
        .filter(|function| function.exported)
        .map(|function| function.name.clone())
        .collect())
}

pub(super) fn provisional(
    file: &SurfaceFile,
    scope: &Scope,
    constants: &TypeEnvironment,
) -> Result<Arc<FunctionMap>, Diagnostic> {
    validate_unique(file, scope)?;
    let definitions = file
        .functions
        .iter()
        .map(|function| definition(file, scope, function))
        .collect::<Result<Vec<_>, _>>()?;
    let context = scope
        .expression_context()
        .with_provisional_values(constants.clone());
    expression::compile_functions(&context, &definitions)
        .map(|compiled| compiled.functions_arc())
        .map_err(|error| diagnostic(file, error))
}

pub(super) fn retain_exports(scope: &mut Scope, names: &BTreeSet<String>) {
    let method_roots = scope.methods.function_ids().collect::<Vec<_>>();
    Arc::make_mut(&mut scope.functions).retain_visible_with_roots(names, method_roots);
}

fn validate_unique(file: &SurfaceFile, scope: &Scope) -> Result<(), Diagnostic> {
    let mut local = BTreeSet::new();
    for function in &file.functions {
        if !crate::name::is_name(&function.name) || BuiltinFunction::parse(&function.name).is_some()
        {
            return Err(Diagnostic::new(
                "PROGRAM_FUNCTION_EXPRESSION",
                &file.path,
                format!("invalid or reserved function name `{}`", function.name),
                function.span,
            ));
        }
        if scope.functions.lookup(&function.name).is_some() || !local.insert(&function.name) {
            return Err(Diagnostic::new(
                "PROGRAM_DUPLICATE_SYMBOL",
                &file.path,
                format!("function `{}` is declared more than once", function.name),
                function.span,
            ));
        }
        validate_parameters(file, function)?;
    }
    Ok(())
}

fn validate_parameters(file: &SurfaceFile, function: &FunctionDecl) -> Result<(), Diagnostic> {
    if function.parameters.len() > MAX_FUNCTION_PARAMETERS {
        return Err(Diagnostic::new(
            "PROGRAM_FUNCTION_PARAMETER_LIMIT",
            &file.path,
            format!(
                "function `{}` exceeds the {MAX_FUNCTION_PARAMETERS} parameter limit",
                function.name
            ),
            function.parameters[MAX_FUNCTION_PARAMETERS].span,
        ));
    }
    let mut names = BTreeSet::new();
    for parameter in &function.parameters {
        if !crate::name::is_name(&parameter.name) || !names.insert(&parameter.name) {
            return Err(Diagnostic::new(
                "PROGRAM_FUNCTION_EXPRESSION",
                &file.path,
                format!(
                    "function `{}` has an invalid or duplicate parameter `{}`",
                    function.name, parameter.name
                ),
                parameter.span,
            ));
        }
    }
    Ok(())
}

fn definition(
    file: &SurfaceFile,
    scope: &Scope,
    function: &FunctionDecl,
) -> Result<FunctionDefinition, Diagnostic> {
    let parameters = function
        .parameters
        .iter()
        .map(|parameter| {
            type_annotations::resolve(file, scope, &parameter.type_syntax).map(|value| {
                let resolved = FunctionParameter::new(&parameter.name, value);
                parameter
                    .default
                    .as_ref()
                    .map_or(resolved.clone(), |default| {
                        resolved.with_default(
                            file.syntax.slice_text(&default.syntax),
                            Some(FunctionOrigin::new(
                                &file.path,
                                default.span.start..default.span.end,
                            )),
                        )
                    })
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let return_type = type_annotations::resolve(file, scope, &function.return_type_syntax)?;
    Ok(FunctionDefinition::new(
        &function.name,
        parameters,
        return_type,
        file.syntax.slice_text(&function.body.syntax),
    )
    .with_origin(
        FunctionOrigin::new(&file.path, function.body.span.start..function.body.span.end)
            .with_syntax(&file.syntax, &function.body.syntax),
    ))
}

fn diagnostic(file: &SurfaceFile, error: expression::ExpressionError) -> Diagnostic {
    let declaration = error
        .function_name()
        .and_then(|name| file.functions.iter().find(|function| function.name == name));
    let span = authored_span(file, &error)
        .or_else(|| declaration.map(|function| body_span(function, error.span())))
        .unwrap_or_default();
    Diagnostic::new(code(error.code()), &file.path, error.to_string(), span)
}

fn authored_span(file: &SurfaceFile, error: &expression::ExpressionError) -> Option<Span> {
    error
        .authored_origin()
        .filter(|origin| origin.source_id() == file.path)
        .and_then(|_| error.authored_span())
        .map(|span| Span {
            start: span.start,
            end: span.end,
        })
}

fn body_span(function: &FunctionDecl, relative: std::ops::Range<usize>) -> Span {
    let start = function.body.span.start.saturating_add(relative.start);
    let end = function.body.span.start.saturating_add(relative.end);
    Span {
        start: start.min(function.body.span.end),
        end: end.min(function.body.span.end),
    }
}

fn code(code: &str) -> &'static str {
    match code {
        "EXPRESSION_FUNCTION_CYCLE" => "PROGRAM_FUNCTION_CYCLE",
        "EXPRESSION_RETURN_TYPE" => "PROGRAM_FUNCTION_RETURN_TYPE",
        "EXPRESSION_RETAINED_LIMIT" => "PROGRAM_RETAINED_SCOPE_BUDGET",
        _ => "PROGRAM_FUNCTION_EXPRESSION",
    }
}
