use std::collections::BTreeMap;

use veac_lang::program::expression::{Effect, Stage};
use veac_lang::program::{
    CompilerDatabase, DomainOperationId, DomainType, LoadedSource, ModuleInterface,
    ModuleInterfaceType, ModuleTypeDefinitionInterface, SourceLoader,
};

#[derive(Default)]
struct Loader {
    sources: BTreeMap<String, String>,
}

impl SourceLoader for Loader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        self.sources
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| format!("missing module `{requested}`"))
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}

fn compile(source: &str) -> ModuleInterface {
    CompilerDatabase::default()
        .module_interface(
            LoadedSource {
                id: "main.veac".to_owned(),
                source: source.to_owned(),
            },
            &Loader::default(),
        )
        .unwrap()
}

#[test]
fn facade_projects_exports_defaults_methods_types_and_constants() {
    let interface = compile(
        r#"module {
  export struct Timing { duration: time, }
  export enum CardPlacement { Center, Corner { x: length, }, }
  impl Timing @timing {
    export fn padded(self, extra: time = 1s) -> time { self.duration + extra }
  }
  export fn identity(value: Timing) -> Timing { value }
  export const time base = 250ms;
}"#,
    );
    assert_eq!(interface.functions.len(), 1);
    assert_eq!(interface.functions[0].name, "identity");
    assert_eq!(interface.methods.len(), 1);
    assert_eq!(interface.methods[0].receiver.source_id, "main.veac");
    assert!(interface.methods[0].parameters[0].has_default);
    assert_eq!(interface.types.len(), 2);
    let ModuleTypeDefinitionInterface::Enum { variants } = &interface.types[0].definition else {
        panic!("alphabetically first Placement export must be an enum")
    };
    assert_eq!(variants[1].fields[0].name, "x");
    assert_eq!(interface.constants[0].name, "base");
}

#[test]
fn imported_interfaces_are_not_reexported() {
    let loader = Loader {
        sources: BTreeMap::from([(
            "dependency.veac".to_owned(),
            r#"module {
  export struct Foreign {}
  impl Foreign @foreign { export fn value(self) -> int { 1 } }
}"#
            .to_owned(),
        )]),
    };
    let interface = CompilerDatabase::default()
        .module_interface(
            LoadedSource {
                id: "main.veac".to_owned(),
                source: r#"module {
  import "./dependency.veac" as dep;
  export fn local(value: dep.Foreign) -> int { value.value() }
}"#
                .to_owned(),
            },
            &loader,
        )
        .unwrap();
    assert_eq!(interface.functions.len(), 1);
    assert_eq!(interface.functions[0].name, "local");
    assert!(matches!(
        &interface.functions[0].parameters[0].value_type,
        ModuleInterfaceType::Named(name)
            if name.source_id == "dependency.veac" && name.name == "Foreign"
    ));
    assert!(interface.types.is_empty());
    assert!(interface.methods.is_empty());
}

#[test]
fn facade_preserves_inferred_effects_stages_and_parameter_dependencies() {
    let interface = compile(
        r#"module {
  export fn identity(value: int) -> int { value }
  export fn mutate(value: int) -> int {
    var current = value; set current = current + 1; current
  }
  export fn make_canvas(width: length, height: length) -> Canvas {
    canvas(width, height)
  }
}"#,
    );
    let identity = &interface.functions[0];
    assert_eq!(identity.name, "identity");
    assert_eq!(identity.semantics.effect, Effect::Pure);
    assert_eq!(identity.semantics.result.shape, Stage::Const);
    let dependency = identity.semantics.result.parameters[0];
    assert!(dependency.shape_from_shape);
    assert!(dependency.leaf_from_leaf);
    assert!(identity.semantics.result.receiver.is_none());

    let canvas = &interface.functions[1];
    assert_eq!(canvas.name, "make_canvas");
    assert_eq!(canvas.semantics.result.shape, Stage::Build);
    assert_eq!(canvas.semantics.result.leaf, Stage::Const);
    assert!(matches!(
        &canvas.return_type,
        ModuleInterfaceType::Domain { name, opcode }
            if name == DomainType::Canvas.name() && *opcode == DomainType::Canvas.opcode()
    ));

    let mutate = &interface.functions[2];
    assert_eq!(mutate.semantics.effect, Effect::LocalMutation);
    assert!(mutate.semantics.contains_local_mutation);
}

#[test]
fn capability_inventory_walks_private_helpers_defaults_and_verified_closures() {
    let interface = compile(
        r#"module {
  fn private_canvas() -> Canvas { canvas(1px, 1px) }
  fn private_rate() -> FrameRate { frame_rate(30, 1) }
  export fn dimensions(value: Canvas = private_canvas()) -> Canvas {
    let build = fn() -> FrameRate effect pure { private_rate() };
    let ignored = build;
    value
  }
}"#,
    );
    assert_eq!(
        interface
            .domain_capabilities
            .iter()
            .map(|value| (value.opcode, value.name.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                DomainOperationId::Canvas.opcode(),
                DomainOperationId::Canvas.name()
            ),
            (
                DomainOperationId::FrameRate.opcode(),
                DomainOperationId::FrameRate.name()
            ),
        ]
    );
}

#[test]
fn facade_rejects_project_entries() {
    let error = CompilerDatabase::default()
        .module_interface(
            LoadedSource {
                id: "main.veac".to_owned(),
                source: "fn main(context: Context) -> Project { project(identifier(\"p\"), project_settings(1)) }".to_owned(),
            },
            &Loader::default(),
        )
        .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_FORMAT_EXPECTED_MODULE");
}

#[path = "module_interface/routing.rs"]
mod routing;
#[path = "module_interface/semantics.rs"]
mod semantics;
