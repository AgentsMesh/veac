use super::{helpers, identity};
use veac_lang::package::api::{ApiExport, ApiPrimitive, ApiSemanticEffect, ApiStage, ApiType};
use veac_lang::package::{discover_package, PackageErrorKind};

fn write_root_source(root: &std::path::Path, source: &str) {
    std::fs::write(root.join("main.veac"), source).unwrap();
    helpers::rewrite_entry_digest(root, source.as_bytes());
}

fn write_dependency_source(root: &std::path::Path, source: &str) {
    std::fs::write(root.join("vendor/util/lib.veac"), source).unwrap();
    helpers::rewrite_dependency_digest(root, source.as_bytes());
}

fn assert_interface_mismatch(root: &std::path::Path, package: &str) {
    let error = discover_package(root).unwrap_err();
    assert_eq!(error.kind(), PackageErrorKind::Contract);
    assert!(
        error
            .message()
            .contains(&format!("package {package} API metadata does not match")),
        "{}",
        error.message()
    );
}

#[test]
fn discovery_accepts_compiler_verified_empty_module_interfaces() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let discovery = discover_package(temp.path()).unwrap();
    assert!(discovery.root.api.exports.is_empty());
    assert!(discovery.dependencies[0].api.exports.is_empty());
}

#[test]
fn discovery_rejects_authored_metadata_that_omits_an_export() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    write_root_source(temp.path(), "module { export fn answer() -> int { 42 } }\n");
    assert_interface_mismatch(temp.path(), "root@1.0.0");
}

#[derive(Clone, Copy)]
enum CallableDrift {
    Effect,
    Stage,
    Type,
    Default,
    Capability,
}

#[test]
fn discovery_rejects_every_compiler_derived_callable_field_drift() {
    for drift in [
        CallableDrift::Effect,
        CallableDrift::Stage,
        CallableDrift::Type,
        CallableDrift::Default,
        CallableDrift::Capability,
    ] {
        let temp = tempfile::tempdir().unwrap();
        helpers::write_contract(temp.path());
        write_root_source(
            temp.path(),
            r#"module {
  export fn make_canvas(width: length = 1px) -> Canvas { canvas(width, 1px) }
}
"#,
        );
        let mut api = helpers::derive_root_api(temp.path());
        if matches!(drift, CallableDrift::Capability) {
            assert!(!api.domain_capabilities.is_empty());
            api.domain_capabilities.clear();
        } else {
            let ApiExport::Function {
                parameters,
                return_type,
                semantics,
                ..
            } = &mut api.exports[0]
            else {
                panic!("fixture must derive a function export")
            };
            match drift {
                CallableDrift::Effect => semantics.effect = ApiSemanticEffect::LocalMutation,
                CallableDrift::Stage => semantics.result.shape = ApiStage::Temporal,
                CallableDrift::Type => {
                    *return_type = ApiType::Primitive {
                        name: ApiPrimitive::Int,
                    }
                }
                CallableDrift::Default => parameters[0].has_default = false,
                CallableDrift::Capability => unreachable!(),
            }
        }
        helpers::rewrite_root_api(temp.path(), &api);
        assert_interface_mismatch(temp.path(), "root@1.0.0");
    }
}

#[test]
fn discovery_rejects_dependency_interface_drift_even_when_unimported() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    write_dependency_source(
        temp.path(),
        "module { export fn duration() -> time { 1s } }\n",
    );
    assert_interface_mismatch(temp.path(), "util@2.0.0");
}

#[test]
fn discovery_rejects_project_entries_for_root_and_dependencies() {
    let project = "fn main(context: Context) -> Project { \
        project(identifier(\"p\"), project_settings(600)) }\n";
    for dependency in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        helpers::write_contract(temp.path());
        if dependency {
            write_dependency_source(temp.path(), project);
        } else {
            write_root_source(temp.path(), project);
        }
        let error = discover_package(temp.path()).unwrap_err();
        assert_eq!(error.kind(), PackageErrorKind::Contract);
        assert!(error
            .message()
            .contains("entry is not a valid exported module"));
    }
}

#[test]
fn authored_dependency_api_drift_is_compared_after_digest_validation() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let source = "module { export fn duration() -> time { 1s } }\n";
    write_dependency_source(temp.path(), source);
    let mut api = helpers::derive_api(
        temp.path().join("vendor/util/lib.veac"),
        identity("util", "2.0.0"),
    );
    let ApiExport::Function { return_type, .. } = &mut api.exports[0] else {
        panic!("fixture must derive a function export")
    };
    *return_type = ApiType::Primitive {
        name: ApiPrimitive::Int,
    };
    helpers::rewrite_dependency_api(temp.path(), &api);
    assert_interface_mismatch(temp.path(), "util@2.0.0");
}
