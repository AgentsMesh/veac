use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use veac_lang::program::{
    format_path, format_source_with_loader, parse_build_input_manifest, prepare_path,
    prepare_with_loader, BuiltProgram, FileSystemLoader, LoadedSource, SourceLoader,
};

fn examples_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn example_sources() -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(examples_root())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path().join("main.veac"))
        .filter(|path| path.is_file())
        .collect();
    paths.sort();
    paths
}

#[test]
fn every_example_builds_to_valid_canonical_ir() {
    let paths = example_sources();
    assert_eq!(paths.len(), 38, "example inventory changed unexpectedly");
    for path in paths {
        let built = build_example(&path).unwrap_or_else(|diagnostics| {
            panic!("{} failed to build: {diagnostics:?}", path.display())
        });
        veac_ir::validate(built.envelope())
            .unwrap_or_else(|diagnostics| panic!("{} invalid: {diagnostics:?}", path.display()));
    }
}

#[test]
fn all_examples_format_idempotently_and_the_formatted_graph_builds() {
    let paths = example_sources();
    assert_eq!(paths.len(), 38, "example inventory changed unexpectedly");
    for entry in paths {
        let original = prepare_path(&entry).unwrap_or_else(|diagnostics| {
            panic!("{} failed to prepare: {diagnostics:?}", entry.display())
        });
        let directory = entry.parent().unwrap();
        let mut formatted = BTreeMap::new();
        for id in original.sources().keys() {
            let path = directory.join(id);
            let once = format_path(&path).unwrap_or_else(|diagnostics| {
                panic!("{} failed to format: {diagnostics:?}", path.display())
            });
            let (loader, mut source) = FileSystemLoader::for_entry(&path).unwrap();
            source.source = once.clone();
            let twice = format_source_with_loader(source, &loader).unwrap();
            assert_eq!(
                once,
                twice,
                "{} is not formatter-idempotent",
                path.display()
            );
            formatted.insert(id.clone(), once);
        }
        build_formatted_example(&entry, formatted);
    }
}

fn build_formatted_example(entry: &Path, sources: BTreeMap<String, String>) {
    let loader = SnapshotLoader { sources };
    let root = LoadedSource {
        id: "main.veac".to_owned(),
        source: loader.sources["main.veac"].clone(),
    };
    let prepared = prepare_with_loader(root, &loader).unwrap_or_else(|diagnostics| {
        panic!(
            "{} formatted graph failed: {diagnostics:?}",
            entry.display()
        )
    });
    let manifest_path = entry.parent().unwrap().join("build-inputs.json");
    let built = if manifest_path.is_file() {
        let json = fs::read_to_string(&manifest_path).unwrap();
        let manifest = parse_build_input_manifest(&json).unwrap();
        prepared.execute_with_inputs(&manifest)
    } else {
        prepared.execute()
    }
    .unwrap_or_else(|diagnostics| {
        panic!(
            "{} formatted source failed: {diagnostics:?}",
            entry.display()
        )
    });
    veac_ir::validate(built.envelope()).unwrap();
}

struct SnapshotLoader {
    sources: BTreeMap<String, String>,
}

impl SourceLoader for SnapshotLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let requested = requested.strip_prefix("./").unwrap_or(requested);
        let id = Path::new(importer)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(requested)
            .to_string_lossy()
            .into_owned();
        self.sources
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| format!("missing snapshot module {requested}"))
    }
}

fn build_example(path: &Path) -> Result<BuiltProgram, veac_lang::program::Diagnostics> {
    let prepared = prepare_path(path)?;
    let manifest_path = path.parent().unwrap().join("build-inputs.json");
    if !manifest_path.is_file() {
        return prepared.execute();
    }
    let json =
        fs::read_to_string(&manifest_path).expect("example Build input manifest is readable");
    let manifest = parse_build_input_manifest(&json)
        .unwrap_or_else(|error| panic!("{} is invalid: {error}", manifest_path.display()));
    prepared.execute_with_inputs(&manifest)
}

#[test]
fn example_directories_have_one_entry_and_only_reachable_modules() {
    for path in example_sources() {
        let directory = path.parent().unwrap();
        let mut sources = fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|value| value == "veac"))
            .collect::<Vec<_>>();
        sources.sort();
        let graph: BTreeSet<_> = prepare_path(&path)
            .unwrap()
            .sources()
            .keys()
            .cloned()
            .collect();
        let files: BTreeSet<_> = sources
            .iter()
            .map(|source| source.file_name().unwrap().to_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            files,
            graph,
            "{} contains an unreachable .veac module",
            directory.display()
        );
        for source in sources.into_iter().filter(|source| source != &path) {
            let text = fs::read_to_string(&source).unwrap();
            assert!(text.trim_start().starts_with("module {"));
        }
    }
}

#[test]
fn gallery_catalog_is_the_complete_example_source_of_truth() {
    let root = examples_root();
    let catalog: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("catalog/gallery.json")).unwrap())
            .unwrap();
    let catalog_sources: BTreeSet<_> = catalog["targets"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|target| target["example"].as_str())
        .map(str::to_owned)
        .collect();
    let filesystem_sources: BTreeSet<_> = example_sources()
        .into_iter()
        .map(|path| format!("examples/{}", path.strip_prefix(&root).unwrap().display()))
        .collect();
    assert_eq!(catalog_sources, filesystem_sources);
}
