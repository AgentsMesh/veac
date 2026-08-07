use std::collections::BTreeSet;

use crate::program::diagnostic::Diagnostic;
use crate::program::model::{SurfaceFile, TypeDecl, TypeDeclKind, TypeFieldDecl};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRef, VariantIndex,
};

pub(super) fn resolve(
    file: &SurfaceFile,
    declaration: &TypeDecl,
    names: &dyn Fn(&str) -> Option<TypeRef>,
) -> Result<TypeDefinition, Diagnostic> {
    let kind = match &declaration.kind {
        TypeDeclKind::Struct(value) => TypeDefinitionKind::Struct(StructDefinition::new(fields(
            file,
            &value.fields,
            "struct field",
            names,
        )?)),
        TypeDeclKind::Enum(value) => {
            if value.variants.is_empty() {
                return Err(Diagnostic::new(
                    "PROGRAM_ENUM_EMPTY",
                    &file.path,
                    "enum must declare at least one variant",
                    declaration.name_span,
                ));
            }
            let mut names_seen = BTreeSet::new();
            let variants = value
                .variants
                .iter()
                .enumerate()
                .map(|(index, variant)| {
                    if !names_seen.insert(&variant.name) {
                        return Err(Diagnostic::new(
                            "PROGRAM_DUPLICATE_ENUM_VARIANT",
                            &file.path,
                            format!("enum variant `{}` is declared more than once", variant.name),
                            variant.name_span,
                        ));
                    }
                    Ok(EnumVariantDefinition::new(
                        VariantIndex::from_position(index).expect("parser bounded variants"),
                        variant.name.clone(),
                        fields(file, &variant.fields, "enum payload field", names)?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            TypeDefinitionKind::Enum(EnumDefinition::new(variants))
        }
    };
    Ok(TypeDefinition::new(
        file.path.as_str(),
        declaration.name.as_str(),
        kind,
    ))
}

fn fields(
    file: &SurfaceFile,
    declarations: &[TypeFieldDecl],
    kind: &str,
    names: &dyn Fn(&str) -> Option<TypeRef>,
) -> Result<Vec<FieldDefinition>, Diagnostic> {
    let mut names_seen = BTreeSet::new();
    declarations
        .iter()
        .enumerate()
        .map(|(index, field)| {
            if !names_seen.insert(&field.name) {
                return Err(Diagnostic::new(
                    "PROGRAM_DUPLICATE_TYPE_FIELD",
                    &file.path,
                    format!("{kind} `{}` is declared more than once", field.name),
                    field.name_span,
                ));
            }
            let value_type = field.type_syntax.resolve(names).map_err(|error| {
                Diagnostic::new(error.code(), &file.path, error.message(), error.span())
            })?;
            Ok(FieldDefinition::new(
                FieldIndex::from_position(index).expect("parser bounded fields"),
                field.name.clone(),
                value_type,
            ))
        })
        .collect()
}
