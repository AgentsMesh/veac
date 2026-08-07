use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn every_example_executes_lowering_in_the_package_unit_target() {
    for path in example_sources() {
        let prepared = crate::program::prepare_path(&path)
            .unwrap_or_else(|errors| panic!("{}: {errors}", path.display()));
        let manifest_path = path.parent().unwrap().join("build-inputs.json");
        let built = if manifest_path.is_file() {
            let source = fs::read_to_string(&manifest_path).unwrap();
            let manifest = crate::program::parse_build_input_manifest(&source).unwrap();
            prepared.execute_with_inputs(&manifest)
        } else {
            prepared.execute()
        }
        .unwrap_or_else(|errors| panic!("{}: {errors}", path.display()));
        assert!(
            veac_ir::validate(built.envelope()).is_ok(),
            "{}",
            path.display()
        );
    }
}

#[test]
fn authored_nested_sink_fixture_executes_in_the_package_unit_target() {
    let source = include_str!("../../../../../tests/fixtures/authored_sink_matrix.veac");
    let built = crate::program::build_source(source).unwrap();
    assert_eq!(built.envelope().temporal.bindings.len(), 19);
    assert!(veac_ir::validate(built.envelope()).is_ok());
}

fn example_sources() -> Vec<PathBuf> {
    let mut paths = fs::read_dir(examples())
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("main.veac"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}
