use std::fs;

use tempfile::tempdir;
use veac_lang::program::{prepare_path, Diagnostics, ExecutableBuild, TypeDefinitionKind, TypeId};

const MODULE: &str = r#"module {
  export struct Brand { title: text, accent: color, }
  export enum CardPlacement {
    Center,
    Corner { x: length, y: length, },
  }
}"#;

#[test]
fn imported_alias_keeps_identity_and_declaration_order_layout() {
    let program = compile(MODULE, "fn inspect(value: types.Brand) -> int { 0 }").unwrap();
    let registry = program.type_registry();
    let brand = registry.resolve("types.Brand").unwrap();
    assert_eq!(brand.id(), TypeId::derive("types.veac", "Brand"));
    assert_eq!(brand.diagnostic_name(), "types.Brand");
    let definition = registry.definition(brand.id()).unwrap();
    let TypeDefinitionKind::Struct(value) = definition.kind() else {
        panic!("Brand must resolve to a struct")
    };
    assert_eq!(
        value
            .fields()
            .iter()
            .map(|field| (field.index().value(), field.name()))
            .collect::<Vec<_>>(),
        [(0, "title"), (1, "accent")]
    );
    let placement = registry.resolve("types.CardPlacement").unwrap();
    let TypeDefinitionKind::Enum(value) = registry.definition(placement.id()).unwrap().kind()
    else {
        panic!("Placement must resolve to an enum")
    };
    assert_eq!(
        value
            .variants()
            .iter()
            .map(|variant| (variant.index().value(), variant.name()))
            .collect::<Vec<_>>(),
        [(0, "Center"), (1, "Corner")]
    );
}

#[test]
fn private_type_names_are_not_imported_and_alias_spelling_survives_diagnostics() {
    let module = r#"module {
      struct Secret { value: text, }
      export struct Brand { value: text, }
    }"#;
    let error = compile(module, "fn inspect(value: types.Secret) -> int { 0 }").unwrap_err();
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_UNKNOWN_TYPE");
    assert_eq!(diagnostic.path, "main.veac");
    assert_eq!(
        diagnostic.span.end - diagnostic.span.start,
        "types.Secret".len()
    );

    let error = compile(module, "fn bad() -> types.Brand { 0 }").unwrap_err();
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, "PROGRAM_FUNCTION_RETURN_TYPE");
    assert!(diagnostic.message.contains("types.Brand"));

    let program = compile(module, "").unwrap();
    assert!(program
        .type_registry()
        .definitions()
        .all(|definition| definition.declared_name() != "Secret"));
}

#[test]
fn every_public_signature_rejects_private_type_leaks() {
    let declarations = [
        "export struct Public { hidden: list<Hidden>, } struct Hidden {}",
        "export fn leak(value: list<Hidden>) -> int { 0 } struct Hidden {}",
        "export fn leak() -> fn(Hidden) -> int effect pure { fn(value: Hidden) -> int effect pure { 0 } } struct Hidden {}",
        "export const Hidden leak = 0; struct Hidden {}",
    ];
    for declaration in declarations {
        let module = format!("module {{ {declaration} }}");
        let error = compile(&module, "").unwrap_err();
        assert_eq!(
            error.as_slice()[0].code,
            "PROGRAM_PRIVATE_TYPE_LEAK",
            "{declaration}"
        );
    }
}

#[test]
fn imported_and_forward_references_are_valid_without_changing_type_id() {
    let module = r#"module {
      export struct Timeline { placement: CardPlacement, }
      export enum CardPlacement { Center, Corner { x: length, }, }
      export fn inspect(value: list<Timeline>) -> int { 0 }
    }"#;
    let program = compile(
        module,
        "fn use(value: types.Timeline) -> int { types.inspect([value]) }",
    )
    .unwrap();
    let timeline = program.type_registry().resolve("types.Timeline").unwrap();
    assert_eq!(timeline.id(), TypeId::derive("types.veac", "Timeline"));
}

fn compile(module: &str, declarations: &str) -> Result<ExecutableBuild, Diagnostics> {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(temp.path().join("types.veac"), module).unwrap();
    fs::write(&entry, entry_source(declarations)).unwrap();
    prepare_path(&entry)
}

fn entry_source(declarations: &str) -> String {
    format!(
        r#"import "types.veac" as types;
{declarations}
fn main(context: Context) -> Project {{
  let timeline = sequence(identifier("main"), "名义类型注册表",
    sequence_settings(canvas(1px, 1px), frame_rate(1, 1), 8000));
  project(identifier("nominal-types"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}}"#
    )
}
