use super::super::expression::{CompiledFunction, FunctionParameter, FunctionSummary};
use super::super::model::Scope;
use super::super::{CompilerSourceRevision, Diagnostics, TypeRegistry};
use super::model::*;

mod types;

use types::{nominal_name, type_interface, value_type};

pub(super) fn scope(
    source_id: String,
    revision: CompilerSourceRevision,
    scope: &Scope,
) -> Result<ModuleInterface, Diagnostics> {
    let functions = scope
        .functions
        .iter()
        .map(|(name, function)| function_interface(name, function, scope.types.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;
    let methods = scope
        .methods
        .definitions()
        .filter(|method| method.owner_source() == source_id)
        .map(|method| method_interface(method, scope))
        .collect::<Result<Vec<_>, _>>()?;
    let types = scope
        .types
        .names()
        .map(|(name, reference)| {
            let definition = scope
                .types
                .definition(reference.id())
                .expect("exported type name has a verified definition");
            type_interface(name, definition, scope.types.as_ref())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let constants = scope
        .values
        .iter()
        .map(|(name, value)| {
            Ok(ModuleConstantInterface {
                name: name.clone(),
                value_type: value_type(&value.value_type(), scope.types.as_ref())?,
            })
        })
        .collect::<Result<Vec<_>, Diagnostics>>()?;
    let domain_capabilities = super::capabilities::collect(&source_id, scope)?;
    Ok(ModuleInterface {
        source_id,
        revision,
        functions,
        methods,
        types,
        constants,
        domain_capabilities,
    })
}

fn function_interface(
    name: &str,
    function: &CompiledFunction,
    types: &TypeRegistry,
) -> Result<ModuleFunctionInterface, Diagnostics> {
    Ok(ModuleFunctionInterface {
        name: name.to_owned(),
        parameters: parameters(function.parameters(), types)?,
        return_type: value_type(function.return_type(), types)?,
        semantics: semantics(function.summary(), function.parameters().len(), false),
    })
}

fn method_interface(
    method: &super::super::MethodDefinition,
    scope: &Scope,
) -> Result<ModuleMethodInterface, Diagnostics> {
    let signature = method.signature();
    let function = scope
        .functions
        .lookup_by_id(signature.function_id())
        .expect("exported method has a compiled function summary");
    Ok(ModuleMethodInterface {
        receiver: nominal_name(signature.receiver().id(), scope.types.as_ref())?,
        name: signature.name().to_owned(),
        parameters: parameters(signature.explicit_parameters(), scope.types.as_ref())?,
        return_type: value_type(signature.return_type(), scope.types.as_ref())?,
        semantics: semantics(
            function.summary(),
            signature.explicit_parameters().len(),
            true,
        ),
    })
}

fn parameters(
    values: &[FunctionParameter],
    types: &TypeRegistry,
) -> Result<Vec<ModuleParameterInterface>, Diagnostics> {
    values
        .iter()
        .map(|value| {
            Ok(ModuleParameterInterface {
                name: value.name.clone(),
                value_type: value_type(&value.value_type, types)?,
                has_default: value.has_default(),
            })
        })
        .collect()
}

fn semantics(
    summary: &FunctionSummary,
    explicit_arity: usize,
    has_receiver: bool,
) -> ModuleCallableSemantics {
    let result = summary.result();
    let offset = usize::from(has_receiver);
    let parameters = (0..explicit_arity)
        .map(|index| dependency(summary, index + offset))
        .collect();
    ModuleCallableSemantics::new(
        summary.effect(),
        summary.contains_local_mutation(),
        ModuleResultSemantics::new(
            result.shape_stage(),
            result.leaf_stage(),
            has_receiver.then(|| dependency(summary, 0)),
            parameters,
        ),
    )
}

fn dependency(summary: &FunctionSummary, index: usize) -> ModuleParameterDependency {
    let result = summary.result();
    ModuleParameterDependency {
        shape_from_shape: result
            .shape_dependencies()
            .depends_on_parameter_shape(index),
        shape_from_leaf: result.shape_dependencies().depends_on_parameter_leaf(index),
        leaf_from_shape: result.leaf_dependencies().depends_on_parameter_shape(index),
        leaf_from_leaf: result.leaf_dependencies().depends_on_parameter_leaf(index),
    }
}
