use super::*;
use crate::executor::tests::support::{bundle, command, protected_resource, video_task};
use veac_codegen::emitter::{
    BackendFilterContract, BackendFilterEscape, BackendResource, MAX_FILTER_GRAPH_BYTES,
};

const FILE: &str = "__VEAC_FILTER_RESOURCE_0000__";
const DIRECTORY: &str = "__VEAC_FILTER_RESOURCE_0001__";

#[test]
fn inputs_luts_and_duplicate_font_names_rebind_to_private_snapshots() {
    let temp = tempfile::tempdir().unwrap();
    let lut = write(temp.path().join("look.cube"), b"lut");
    let first = write(temp.path().join("a/font.ttf"), b"font-a");
    let second = write(temp.path().join("b/font.ttf"), b"font-b");
    let output = temp.path().join("output.bin");
    let mut task = video_task("snapshot", &output);
    let contract = filter_contract(&lut, &[first.clone(), second.clone()]);
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command.inputs.push(BackendInput { path: lut.clone() });
    command.filter_graph = Some(contract.render_original().unwrap());
    command.filter_contract = Some(contract);
    let mut value = bundle(vec![task.clone()]);
    value.protected_resources = resources(&[&lut, &first, &second]);

    let snapshots = capture(&value, deadline()).unwrap();
    let root = snapshots._directory.path().to_path_buf();
    let rebound = snapshots.rebind(&task, deadline()).unwrap();
    let BackendAction::Ffmpeg(command) = rebound.action else {
        unreachable!()
    };
    assert_ne!(command.inputs[0].path, lut);
    assert!(command.inputs[0].path.starts_with(&root));
    let graph = command.filter_graph.unwrap();
    for original in [&lut, &first, &second] {
        assert!(!graph.contains(original.to_str().unwrap()), "{graph}");
    }
    let directory = snapshots.directories.values().next().unwrap();
    let entries = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 2);
    assert_ne!(entries[0].file_name(), entries[1].file_name());
    let mut bytes = entries
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .collect::<Vec<_>>();
    bytes.sort();
    assert_eq!(bytes, [b"font-a".to_vec(), b"font-b".to_vec()]);
    drop(snapshots);
    assert!(!root.exists());
}

#[test]
fn rebound_filter_graph_rechecks_the_total_graph_budget() {
    let temp = tempfile::tempdir().unwrap();
    let resource = write(temp.path().join("r"), b"resource");
    let mut value = bundle(vec![video_task("budget", &temp.path().join("out"))]);
    value.protected_resources = resources(&[&resource]);
    let snapshots = capture(&value, deadline()).unwrap();
    let base = BackendFilterContract::new(
        FILE.to_owned(),
        vec![BackendFilterBinding::file(
            FILE.to_owned(),
            resource.clone(),
            BackendFilterEscape::Quoted,
        )],
    )
    .unwrap();
    let original = base.render_original().unwrap();
    let bound = base
        .render_bound(&snapshots.files, &BTreeMap::new())
        .unwrap();
    assert!(bound.len() > original.len());
    let template = format!(
        "{}{FILE}",
        "x".repeat(MAX_FILTER_GRAPH_BYTES - original.len())
    );
    let contract = BackendFilterContract::new(template, base.bindings().to_vec()).unwrap();
    let mut task = video_task("budget", &temp.path().join("out"));
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command.filter_graph = Some(contract.render_original().unwrap());
    command.filter_contract = Some(contract);
    assert!(snapshots
        .rebind(&task, deadline())
        .unwrap_err()
        .message
        .contains("graph limit"));
}

#[test]
fn missing_snapshot_and_wrong_identity_report_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");
    let task = {
        let mut command = command(&temp.path().join("out"));
        command.inputs.push(BackendInput { path: missing });
        let mut task = video_task("missing", &temp.path().join("out"));
        task.action = BackendAction::Ffmpeg(command);
        task
    };
    let snapshots = capture(&bundle(Vec::new()), deadline()).unwrap();
    assert!(snapshots.rebind(&task, deadline()).is_err());

    let source = write(temp.path().join("source"), b"source");
    let mut value = bundle(Vec::new());
    let mut resource = protected_resource(&source);
    resource.expected_identity.digest = "0".repeat(64);
    value.protected_resources.push(resource);
    assert!(capture(&value, deadline())
        .unwrap_err()
        .message
        .contains("cannot snapshot protected resource"));
}

fn filter_contract(lut: &Path, fonts: &[PathBuf]) -> BackendFilterContract {
    BackendFilterContract::new(
        format!("lut='{FILE}';fonts='{DIRECTORY}'"),
        vec![
            BackendFilterBinding::file(
                FILE.to_owned(),
                lut.to_path_buf(),
                BackendFilterEscape::Quoted,
            ),
            BackendFilterBinding::directory(
                DIRECTORY.to_owned(),
                fonts[0].parent().unwrap().to_path_buf(),
                fonts.to_vec(),
                BackendFilterEscape::Quoted,
            ),
        ],
    )
    .unwrap()
}

fn resources(paths: &[&Path]) -> Vec<BackendResource> {
    let mut values = paths
        .iter()
        .map(|path| protected_resource(path))
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.path.cmp(&right.path));
    values
}

fn write(path: PathBuf, bytes: &[u8]) -> PathBuf {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

fn deadline() -> Instant {
    Instant::now() + std::time::Duration::from_secs(10)
}

#[test]
fn snapshot_error_helpers_preserve_failure_kinds() {
    let error = snapshot_error(std::io::Error::other("snapshot I/O failed"));
    assert!(error.to_string().contains("snapshot I/O failed"));
}
