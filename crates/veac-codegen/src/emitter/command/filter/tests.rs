use super::*;

const FIRST: &str = "__VEAC_FILTER_RESOURCE_0000__";
const SECOND: &str = "__VEAC_FILTER_RESOURCE_0001__";

#[test]
fn tokens_must_be_unique_and_occur_exactly_once() {
    let duplicate_token = BackendFilterContract::new(
        format!("{FIRST};{SECOND}"),
        vec![file(FIRST, "a"), file(FIRST, "b")],
    )
    .unwrap_err();
    assert!(duplicate_token.contains("unique"));

    for template in ["plain".to_owned(), format!("{FIRST};{FIRST}")] {
        let error = BackendFilterContract::new(template, vec![file(FIRST, "a")]).unwrap_err();
        assert!(error.contains("exactly once"));
    }
}

#[test]
fn templates_may_not_contain_an_unbound_resource_token() {
    let error = BackendFilterContract::new(format!("{FIRST};{SECOND}"), vec![file(FIRST, "a")])
        .unwrap_err();
    assert!(error.contains("unbound token"));
}

#[test]
fn malformed_and_truncated_resource_markers_fail_closed() {
    for template in [
        "__VEAC_FILTER_RESOURCE_X__",
        "__VEAC_FILTER_RESOURCE_000",
        "__VEAC_FILTER_RESOURCE_00000__",
    ] {
        let error = BackendFilterContract::new(template.to_owned(), Vec::new()).unwrap_err();
        assert!(error.contains("malformed token"), "{error}");
    }
}

#[test]
fn rendered_paths_may_contain_text_that_looks_like_a_resource_token() {
    let original = PathBuf::from("original");
    let injected = PathBuf::from("/tmp/__VEAC_FILTER_RESOURCE_9999__/lut.cube");
    let files = BTreeMap::from([(original.clone(), injected.clone())]);
    let rendered = contract(original, BackendFilterEscape::Quoted)
        .render_bound(&files, &BTreeMap::new())
        .unwrap();
    assert!(rendered.contains("__VEAC_FILTER_RESOURCE_9999__"));
}

#[test]
fn replacements_only_target_token_spans_from_the_original_template() {
    let first = PathBuf::from("first");
    let second = PathBuf::from("second");
    let injected = PathBuf::from(format!("/tmp/{FIRST}/{SECOND}"));
    let files = BTreeMap::from([
        (first.clone(), injected.clone()),
        (second.clone(), PathBuf::from("bound-second")),
    ]);
    let contract = BackendFilterContract::new(
        format!("{FIRST};{SECOND}"),
        vec![
            BackendFilterBinding::file(FIRST.to_owned(), first, BackendFilterEscape::Quoted),
            BackendFilterBinding::file(SECOND.to_owned(), second, BackendFilterEscape::Quoted),
        ],
    )
    .unwrap();

    let rendered = contract.render_bound(&files, &BTreeMap::new()).unwrap();
    assert_eq!(rendered, format!("{};bound-second", injected.display()));
}

#[test]
fn contract_accessors_and_original_rendering_preserve_typed_resources() {
    let template = format!("{FIRST};{SECOND}");
    let contract = BackendFilterContract::new(
        template.clone(),
        vec![
            file(FIRST, "lut.cube"),
            BackendFilterBinding::directory(
                SECOND.to_owned(),
                PathBuf::from("fonts"),
                vec![PathBuf::from("font.ttf")],
                BackendFilterEscape::Quoted,
            ),
        ],
    )
    .unwrap();

    assert_eq!(contract.template(), template);
    assert_eq!(contract.bindings().len(), 2);
    assert_eq!(contract.render_original().unwrap(), "lut.cube;fonts");
}

#[test]
fn directory_bindings_must_declare_at_least_one_file() {
    let binding = BackendFilterBinding::directory(
        FIRST.to_owned(),
        PathBuf::from("fonts"),
        Vec::new(),
        BackendFilterEscape::FilterValue,
    );
    let error = BackendFilterContract::new(FIRST.to_owned(), vec![binding]).unwrap_err();
    assert!(error.contains("at least one file"));
}

#[test]
fn bound_rendering_fails_when_a_file_or_directory_is_missing() {
    let file_contract =
        BackendFilterContract::new(FIRST.to_owned(), vec![file(FIRST, "lut")]).unwrap();
    let error = file_contract
        .render_bound(&BTreeMap::new(), &BTreeMap::new())
        .unwrap_err();
    assert!(error.contains("no verified path binding"));

    let files = vec![PathBuf::from("font.ttf")];
    let directory = BackendFilterBinding::directory(
        FIRST.to_owned(),
        PathBuf::from("fonts"),
        files,
        BackendFilterEscape::FilterValue,
    );
    let contract = BackendFilterContract::new(FIRST.to_owned(), vec![directory]).unwrap();
    let error = contract
        .render_bound(&BTreeMap::new(), &BTreeMap::new())
        .unwrap_err();
    assert!(error.contains("no verified path binding"));
}

#[test]
fn bound_paths_use_the_declared_escape_layer() {
    let original = PathBuf::from("original");
    let bound = PathBuf::from("x:y,z");
    let files = BTreeMap::from([(original.clone(), bound)]);
    let directories = BTreeMap::new();

    let option = contract(original.clone(), BackendFilterEscape::FilterValue)
        .render_bound(&files, &directories)
        .unwrap();
    let quoted = contract(original, BackendFilterEscape::Quoted)
        .render_bound(&files, &directories)
        .unwrap();

    assert_eq!(option, r"x\\:y\,z");
    assert_eq!(quoted, r"x\:y\,z");
}

#[test]
fn bound_directory_uses_the_verified_directory_path() {
    let original = PathBuf::from("fonts");
    let binding = BackendFilterBinding::directory(
        FIRST.to_owned(),
        original.clone(),
        vec![PathBuf::from("font.ttf")],
        BackendFilterEscape::FilterValue,
    );
    let contract = BackendFilterContract::new(FIRST.to_owned(), vec![binding]).unwrap();
    let directories = BTreeMap::from([(
        vec![PathBuf::from("font.ttf")],
        PathBuf::from("verified-fonts"),
    )]);

    assert_eq!(
        contract
            .render_bound(&BTreeMap::new(), &directories)
            .unwrap(),
        "verified-fonts"
    );
}

fn contract(path: PathBuf, escape: BackendFilterEscape) -> BackendFilterContract {
    let binding = BackendFilterBinding::file(FIRST.to_owned(), path, escape);
    BackendFilterContract::new(FIRST.to_owned(), vec![binding]).unwrap()
}

fn file(token: &str, path: &str) -> BackendFilterBinding {
    BackendFilterBinding::file(
        token.to_owned(),
        PathBuf::from(path),
        BackendFilterEscape::FilterValue,
    )
}
