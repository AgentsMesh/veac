use crate::source_edit::{
    ImportSource, SourceEditError, SourceImportRef, SourceModuleAnchor, SourceNodeKind,
    SourceNodeRef,
};

pub(super) fn validate_declaration_source(source: &str) -> Result<(), SourceEditError> {
    super::fragment::validate(source)
        .map_err(|message| SourceEditError::InvalidTopLevelDeclaration(message.to_owned()))?;
    crate::program::validate_top_level_fragment(source)
        .map_err(SourceEditError::InvalidTopLevelDeclaration)
}

pub(super) fn validate_declaration_target(target: &SourceNodeRef) -> Result<(), SourceEditError> {
    super::validate_target(target)?;
    if matches!(
        target.kind(),
        SourceNodeKind::Input
            | SourceNodeKind::Constant
            | SourceNodeKind::Function
            | SourceNodeKind::Implementation
            | SourceNodeKind::Struct
            | SourceNodeKind::Enum
            | SourceNodeKind::Temporal
    ) {
        Ok(())
    } else {
        Err(SourceEditError::IncompatibleTopLevelDeclarationTarget)
    }
}

pub(super) fn validate_import_equals(
    target: &SourceImportRef,
    value: &ImportSource,
) -> Result<(), SourceEditError> {
    validate_import_target(target)?;
    validate_import_source(value)?;
    if target.alias == value.alias {
        Ok(())
    } else {
        Err(SourceEditError::InvalidImport(
            "import_equals target and import aliases differ".to_owned(),
        ))
    }
}

pub(super) fn validate_import_source(value: &ImportSource) -> Result<(), SourceEditError> {
    if value.path.is_empty() || value.path.len() > 4_096 || value.path.contains('\0') {
        return Err(SourceEditError::InvalidImport(
            "import path must contain 1..4096 non-NUL bytes".to_owned(),
        ));
    }
    if !crate::name::is_name(&value.alias) {
        return Err(SourceEditError::InvalidImport(format!(
            "invalid import alias {:?}",
            value.alias
        )));
    }
    crate::program::validate_import_fragment(&value.render())
        .map_err(SourceEditError::InvalidImport)
}

pub(super) fn validate_import_target(target: &SourceImportRef) -> Result<(), SourceEditError> {
    crate::source_edit::validate_module_path(&target.module)?;
    if crate::name::is_name(&target.alias) {
        Ok(())
    } else {
        Err(SourceEditError::InvalidImport(format!(
            "invalid import alias {:?}",
            target.alias
        )))
    }
}

pub(super) fn validate_module_anchor(
    module: &str,
    anchor: &SourceModuleAnchor,
) -> Result<(), SourceEditError> {
    crate::source_edit::validate_module_path(module)?;
    let target_module = match anchor {
        SourceModuleAnchor::ModuleStart | SourceModuleAnchor::ModuleEnd => return Ok(()),
        SourceModuleAnchor::BeforeDeclaration { target }
        | SourceModuleAnchor::AfterDeclaration { target } => {
            validate_declaration_target(target)?;
            &target.module
        }
        SourceModuleAnchor::BeforeImport { target }
        | SourceModuleAnchor::AfterImport { target } => {
            validate_import_target(target)?;
            &target.module
        }
    };
    if target_module == module {
        Ok(())
    } else {
        Err(SourceEditError::AnchorModuleMismatch {
            module: module.to_owned(),
            anchor_module: target_module.clone(),
        })
    }
}
