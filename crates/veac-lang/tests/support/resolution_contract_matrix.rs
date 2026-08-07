use std::collections::BTreeMap;

use crate::program::expression::PrimitiveType;
use crate::program::{
    prepare_source, prepare_with_loader, LoadedSource, SourceLoader, TypeSyntax, TypeSyntaxKind,
};

const MAIN: &str = r#"
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), "解析契约",
    sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000));
  project(identifier("resolution"), project_settings(60))
    .with_sequence(timeline).entry(timeline)
}"#;

fn project(declarations: &str) -> String {
    format!("{declarations}\n{MAIN}")
}

fn source_error(declarations: &str) -> &'static str {
    prepare_source(&project(declarations))
        .unwrap_err()
        .as_slice()[0]
        .code
}

#[derive(Default)]
struct Modules(BTreeMap<String, String>);

impl SourceLoader for Modules {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        self.0
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| format!("missing {requested}"))
    }
}

fn loader_error(imports: &str, loader: &dyn SourceLoader) -> &'static str {
    prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: project(imports),
        },
        loader,
    )
    .unwrap_err()
    .as_slice()[0]
        .code
}

#[test]
fn constants_report_cycles_types_runtime_failures_and_duplicates() {
    let cases = [
        (
            "const int first = second; const int second = first;",
            "PROGRAM_CONST_CYCLE",
        ),
        ("const int wrong = true;", "PROGRAM_CONST_TYPE"),
        ("const int broken = 1 / 0;", "PROGRAM_CONST_EXPRESSION"),
        (
            "const int duplicate = 1; const int duplicate = 2;",
            "PROGRAM_DUPLICATE_SYMBOL",
        ),
    ];
    for (source, code) in cases {
        assert_eq!(source_error(source), code, "{source}");
    }
}

#[test]
fn exported_nominal_abi_retains_nested_types_methods_functions_and_constants() {
    let module = r#"module {
      export struct Timing { value: time, }
      export enum Choice { Empty, Value { timing: Timing, } }
      export const time offset = 1s;
      impl Timing @timing { export fn padded(self, extra: time) -> time { self.value + extra } }
      export fn make(value: time) -> Timing { Timing { value: value } }
      export fn inspect(value: fn(list<(Timing, map<text, Choice>)>) -> range<int> effect pure) -> int { 1 }
    }"#;
    let loader = Modules(BTreeMap::from([(
        "types.veac".to_owned(),
        module.to_owned(),
    )]));
    let imports = r#"import "./types.veac" as types;
      const time total = types.offset;
      fn use() -> time { types.make(1s).padded(1s) }"#;
    prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: project(imports),
        },
        &loader,
    )
    .unwrap();
}

#[test]
fn loader_rejects_alias_cycles_non_modules_and_source_id_collisions() {
    let duplicate = Modules(BTreeMap::from([
        ("a.veac".to_owned(), "module {}".to_owned()),
        ("b.veac".to_owned(), "module {}".to_owned()),
    ]));
    assert_eq!(
        loader_error(
            "import \"./a.veac\" as same; import \"./b.veac\" as same;",
            &duplicate,
        ),
        "PROGRAM_IMPORT_ALIAS"
    );

    let cycle = Modules(BTreeMap::from([
        (
            "a.veac".to_owned(),
            "module { import \"./b.veac\" as b; }".to_owned(),
        ),
        (
            "b.veac".to_owned(),
            "module { import \"./a.veac\" as a; }".to_owned(),
        ),
    ]));
    assert_eq!(
        loader_error("import \"./a.veac\" as a;", &cycle),
        "PROGRAM_IMPORT_CYCLE"
    );

    let project_module = Modules(BTreeMap::from([("other.veac".to_owned(), MAIN.to_owned())]));
    assert_eq!(
        loader_error("import \"./other.veac\" as other;", &project_module),
        "PROGRAM_IMPORT_PROJECT"
    );
    assert_eq!(
        loader_error(
            "import \"./one.veac\" as one; import \"./two.veac\" as two;",
            &Collision,
        ),
        "PROGRAM_SOURCE_ID_COLLISION"
    );
}

struct Collision;

impl SourceLoader for Collision {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let suffix = if requested.contains("two") {
            " const int changed = 1;"
        } else {
            ""
        };
        Ok(LoadedSource {
            id: "shared.veac".to_owned(),
            source: format!("module {{{suffix}}}"),
        })
    }
}

#[test]
fn public_type_syntax_formats_every_composite_shape_and_errors() {
    let span = Default::default();
    let integer = TypeSyntax::new(TypeSyntaxKind::Primitive(PrimitiveType::Integer), span);
    let named = TypeSyntax::new(TypeSyntaxKind::Named("Timing".into()), span);
    let tuple = TypeSyntax::new(
        TypeSyntaxKind::Tuple(vec![integer.clone(), named.clone()].into()),
        span,
    );
    let list = TypeSyntax::new(TypeSyntaxKind::List(tuple.into()), span);
    let function = TypeSyntax::new(
        TypeSyntaxKind::Function {
            parameters: vec![list].into(),
            result: TypeSyntax::new(TypeSyntaxKind::Range(integer.into()), span).into(),
            effect: crate::program::expression::FunctionEffect::Pure,
        },
        span,
    );
    assert_eq!(
        function.to_string(),
        "fn(list<(int, Timing)>) -> range<int> effect pure"
    );
    let error = named.resolve(&|_| None).unwrap_err();
    assert_eq!(error.code(), "PROGRAM_UNKNOWN_TYPE");
    assert!(error.to_string().contains("Timing"));
}
