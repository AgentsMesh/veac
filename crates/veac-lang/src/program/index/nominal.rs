use crate::program::diagnostic::Diagnostic;
use crate::program::model::{EnumDecl, StructDecl, SurfaceFile, TypeDecl, TypeDeclKind};
use crate::source_edit::{DeclarationSite, SourceNodeRef};

use super::SourceIndex;

pub(super) fn index(index: &mut SourceIndex, file: &SurfaceFile) -> Result<(), Diagnostic> {
    for declaration in &file.types {
        match &declaration.kind {
            TypeDeclKind::Struct(value) => structure(index, file, declaration, value)?,
            TypeDeclKind::Enum(value) => enumeration(index, file, declaration, value)?,
        }
    }
    Ok(())
}

fn structure(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    declaration: &TypeDecl,
    value: &StructDecl,
) -> Result<(), Diagnostic> {
    let target = SourceNodeRef::structure(&file.path, &declaration.name);
    register_declaration(
        index,
        file,
        target,
        DeclarationSite::StructDeclaration,
        declaration.span,
    )?;
    for field in &value.fields {
        let target = SourceNodeRef::struct_field(&file.path, &declaration.name, &field.name);
        register_declaration(
            index,
            file,
            target,
            DeclarationSite::StructFieldDeclaration,
            field.span,
        )?;
    }
    Ok(())
}

fn enumeration(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    declaration: &TypeDecl,
    value: &EnumDecl,
) -> Result<(), Diagnostic> {
    let target = SourceNodeRef::enumeration(&file.path, &declaration.name);
    register_declaration(
        index,
        file,
        target,
        DeclarationSite::EnumDeclaration,
        declaration.span,
    )?;
    for variant in &value.variants {
        let target = SourceNodeRef::enum_variant(&file.path, &declaration.name, &variant.name);
        register_declaration(
            index,
            file,
            target,
            DeclarationSite::EnumVariantDeclaration,
            variant.span,
        )?;
        for field in &variant.fields {
            let target = SourceNodeRef::enum_variant_field(
                &file.path,
                &declaration.name,
                &variant.name,
                &field.name,
            );
            register_declaration(
                index,
                file,
                target,
                DeclarationSite::EnumVariantFieldDeclaration,
                field.span,
            )?;
        }
    }
    Ok(())
}

fn register_declaration(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    site: DeclarationSite,
    span: crate::authoring::Span,
) -> Result<(), Diagnostic> {
    index.register(&file.path, target.clone(), span)?;
    index.insert_declaration(&file.path, target, site, &file.source, span)
}

#[cfg(test)]
#[path = "nominal_tests.rs"]
mod tests;
