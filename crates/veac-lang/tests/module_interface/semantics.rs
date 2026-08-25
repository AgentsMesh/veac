use super::*;
use veac_lang::program::expression::FunctionEffect;

fn exported_function<'a>(
    interface: &'a ModuleInterface,
    name: &str,
) -> &'a veac_lang::program::ModuleFunctionInterface {
    interface
        .functions
        .iter()
        .find(|function| function.name == name)
        .unwrap()
}

#[test]
fn exported_method_semantics_separate_receiver_from_explicit_parameters() {
    let interface = compile(
        r#"module {
  export struct Timing { duration: time, }
  impl Timing @timing {
    export fn preserve(self, ignored: int) -> Timing { self }
  }
}"#,
    );
    let method = &interface.methods[0];
    let receiver = method.semantics.result.receiver.unwrap();
    assert!(receiver.shape_from_shape);
    assert!(!receiver.shape_from_leaf);
    assert!(!receiver.leaf_from_shape);
    assert!(receiver.leaf_from_leaf);
    assert_eq!(method.semantics.result.parameters.len(), 1);
    assert_eq!(
        method.semantics.result.parameters[0],
        veac_lang::program::ModuleParameterDependency {
            shape_from_shape: false,
            shape_from_leaf: false,
            leaf_from_shape: false,
            leaf_from_leaf: false,
        }
    );
}

#[test]
fn range_results_preserve_all_four_parameter_dependency_axes() {
    let interface = compile(
        r#"module {
  export fn values(start: int, end: int, step: int) -> range<int> {
    start .. end by step
  }
}"#,
    );
    let dependencies = &interface.functions[0].semantics.result.parameters;
    for index in [0, 2] {
        assert!(dependencies[index].shape_from_shape);
        assert!(dependencies[index].shape_from_leaf);
        assert!(dependencies[index].leaf_from_shape);
        assert!(dependencies[index].leaf_from_leaf);
    }
    assert!(dependencies[1].shape_from_shape);
    assert!(dependencies[1].shape_from_leaf);
    assert!(!dependencies[1].leaf_from_shape);
    assert!(!dependencies[1].leaf_from_leaf);
}

#[test]
fn compiler_interface_preserves_declared_function_type_effects() {
    let interface = compile(
        r#"module {
  export fn retain(callback: fn(int) -> int effect emit)
      -> fn(int) -> int effect emit { callback }
}"#,
    );
    let function = exported_function(&interface, "retain");
    assert!(matches!(
        &function.parameters[0].value_type,
        ModuleInterfaceType::Function {
            effect: FunctionEffect::Emit,
            ..
        }
    ));
    assert!(matches!(
        &function.return_type,
        ModuleInterfaceType::Function {
            effect: FunctionEffect::Emit,
            ..
        }
    ));
}

#[test]
fn graph_emitting_exports_report_effect_and_capability() {
    let interface = compile(
        r#"module {
  export fn make_project() -> Project {
    project(identifier("module-project"), project_settings(600))
  }
}"#,
    );
    let function = exported_function(&interface, "make_project");
    assert_eq!(function.semantics.effect, Effect::GraphEmit);
    assert!(!function.semantics.contains_local_mutation);
    assert_eq!(function.semantics.result.shape, Stage::Build);
    assert!(interface
        .domain_capabilities
        .iter()
        .any(|capability| capability.opcode == DomainOperationId::Project.opcode()));
}
