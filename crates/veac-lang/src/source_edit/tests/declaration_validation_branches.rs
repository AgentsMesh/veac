use super::*;

#[test]
fn build_input_and_temporal_declarations_use_their_closed_parsers() {
    let cases = [
        (
            SourceNodeRef::input("main.veac", "base_span"),
            DeclarationSite::BuildInputDeclaration,
            "input parameter base_span: time;",
        ),
        (
            SourceNodeRef::temporal(
                "main.veac",
                "demo",
                "main",
                "visual",
                "hero",
                SourceTemporalProperty::VisualOpacity,
            ),
            DeclarationSite::TemporalDeclaration,
            "animate visual-opacity on clip(@demo, @main, @visual, @hero) { progress }",
        ),
    ];
    for (target, site, source) in cases {
        assert_eq!(
            validate_source_edit_contract(&declaration(target, site, source)),
            Ok(())
        );
    }
}

#[test]
fn temporal_and_build_input_sites_reject_other_declaration_shapes() {
    let cases = [
        (
            SourceNodeRef::input("main.veac", "base_span"),
            DeclarationSite::BuildInputDeclaration,
            "const time base_span = 1s;",
        ),
        (
            SourceNodeRef::temporal(
                "main.veac",
                "demo",
                "main",
                "visual",
                "hero",
                SourceTemporalProperty::VisualOpacity,
            ),
            DeclarationSite::TemporalDeclaration,
            "const scalar opacity = 1.0;",
        ),
    ];
    for (target, site, source) in cases {
        assert!(matches!(
            validate_source_edit_contract(&declaration(target, site, source)),
            Err(SourceEditError::InvalidDeclaration(_))
        ));
    }
}

#[test]
fn structural_targets_reject_invalid_aliases_modules_and_empty_fragments() {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::RemoveImport {
        target: SourceImportRef::new("main.veac", "bad.alias"),
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidImport(_))
    ));

    value.operations = vec![SourceEditOperation::InsertDeclaration {
        module: "../escape.veac".into(),
        anchor: SourceModuleAnchor::ModuleEnd,
        declaration: TopLevelDeclarationSource {
            source: "const time duration = 1s;".into(),
        },
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidModulePath(_))
    ));

    value.operations = vec![SourceEditOperation::SetTopLevelDeclaration {
        target: SourceNodeRef::function("main.veac", "duration"),
        declaration: TopLevelDeclarationSource {
            source: String::new(),
        },
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidTopLevelDeclaration(_))
    ));
}

#[test]
fn expression_validation_accepts_wrapped_pure_symbols_and_rejects_bad_syntax() {
    let mut value = batch();
    value.operations = vec![operation("${duration}")];
    assert_eq!(validate_source_edit_contract(&value), Ok(()));

    value.operations = vec![operation("${duration(}")];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidExpression(_))
    ));
}

fn declaration(target: SourceNodeRef, site: DeclarationSite, source: &str) -> SourceEditBatch {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::SetDeclaration {
        target,
        site,
        declaration: DeclarationSource {
            source: source.into(),
        },
    }];
    value
}
