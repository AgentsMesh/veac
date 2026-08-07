use crate::program::TypeDeclarationFragmentKind as FragmentKind;
use crate::source_edit::{DeclarationSite, SourceEditError, SourceNodeRef};

pub(super) fn validate(
    target: &SourceNodeRef,
    site: DeclarationSite,
    source: &str,
) -> Result<(), SourceEditError> {
    super::validate_target(target)?;
    if !site.accepts_target(target) {
        return Err(SourceEditError::IncompatibleDeclarationSite);
    }
    super::fragment::validate(source)
        .map_err(|message| SourceEditError::InvalidDeclaration(message.to_owned()))?;
    if site == DeclarationSite::BuildInputDeclaration {
        crate::program::validate_build_input_fragment(source)
            .map_err(SourceEditError::InvalidDeclaration)
    } else if site == DeclarationSite::TemporalDeclaration {
        crate::program::validate_temporal_fragment(source)
            .map_err(SourceEditError::InvalidDeclaration)
    } else if matches!(site, DeclarationSite::ComponentAnimation { .. }) {
        crate::program::expression::validate_temporal_attachment(source)
            .map_err(|error| SourceEditError::InvalidDeclaration(error.to_string()))
    } else {
        crate::program::validate_type_declaration_fragment(source, fragment_kind(site).unwrap())
            .map_err(SourceEditError::InvalidDeclaration)
    }
}

fn fragment_kind(site: DeclarationSite) -> Option<FragmentKind> {
    Some(match site {
        DeclarationSite::BuildInputDeclaration => return None,
        DeclarationSite::TemporalDeclaration => return None,
        DeclarationSite::ComponentAnimation { .. } => return None,
        DeclarationSite::StructDeclaration => FragmentKind::Struct,
        DeclarationSite::StructFieldDeclaration => FragmentKind::StructField,
        DeclarationSite::EnumDeclaration => FragmentKind::Enum,
        DeclarationSite::EnumVariantDeclaration => FragmentKind::EnumVariant,
        DeclarationSite::EnumVariantFieldDeclaration => FragmentKind::EnumVariantField,
    })
}
