use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{PrimitiveType, ValueTypeKind};
use crate::program::model::{Scope, SurfaceFile};
use crate::program::{BuildInputDeclaration, TypeDefinitionKind, TypeRegistry, MAX_BUILD_INPUTS};

use super::type_annotations;

pub(super) fn resolve(file: &SurfaceFile, scope: &mut Scope) -> Result<(), Diagnostic> {
    if file.inputs.len() > MAX_BUILD_INPUTS {
        return Err(Diagnostic::new(
            "PROGRAM_INPUT_LIMIT",
            &file.path,
            format!("entry exceeds the {MAX_BUILD_INPUTS} Build input limit"),
            file.inputs[MAX_BUILD_INPUTS].span,
        ));
    }
    let mut names = BTreeSet::new();
    let mut inputs = BTreeMap::new();
    for declaration in &file.inputs {
        if !names.insert(&declaration.name) || collides(file, scope, &declaration.name) {
            return Err(Diagnostic::new(
                "PROGRAM_DUPLICATE_SYMBOL",
                &file.path,
                format!(
                    "Build input `{}` is declared more than once",
                    declaration.name
                ),
                declaration.name_span,
            ));
        }
        let value_type = type_annotations::resolve(file, scope, &declaration.type_syntax)?;
        if !supported(&value_type, &scope.types) {
            return Err(Diagnostic::new(
                "PROGRAM_INPUT_TYPE",
                &file.path,
                "Build inputs require a supported primitive or a payloadless nominal enum",
                declaration.name_span,
            ));
        }
        let input = BuildInputDeclaration::new(
            &file.path,
            declaration.name.clone(),
            declaration.role,
            value_type,
        );
        inputs.insert(declaration.name.clone(), input);
    }
    scope.build_inputs = Arc::new(inputs);
    Ok(())
}

fn collides(file: &SurfaceFile, scope: &Scope, name: &str) -> bool {
    scope.values.contains_key(name)
        || scope.functions.lookup(name).is_some()
        || scope.types.resolve(name).is_some()
        || file.constants.iter().any(|value| value.name == name)
        || file.functions.iter().any(|value| value.name == name)
        || file.types.iter().any(|value| value.name == name)
}

fn supported(value: &crate::program::expression::ValueType, types: &TypeRegistry) -> bool {
    if matches!(
        value.kind(),
        ValueTypeKind::Primitive(
            PrimitiveType::Boolean
                | PrimitiveType::Integer
                | PrimitiveType::Scalar
                | PrimitiveType::Text
                | PrimitiveType::Time
                | PrimitiveType::Length
                | PrimitiveType::Angle
                | PrimitiveType::Color
        )
    ) {
        return true;
    }
    let ValueTypeKind::Nominal(reference) = value.kind() else {
        return false;
    };
    matches!(types.definition(reference.id()).map(|value| value.kind()),
        Some(TypeDefinitionKind::Enum(layout))
            if layout.variants().iter().all(|variant| variant.fields().is_empty()))
}
