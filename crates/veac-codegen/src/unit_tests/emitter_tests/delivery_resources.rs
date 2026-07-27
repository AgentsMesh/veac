use veac_codegen::emitter::{emit_all, CodegenErrorKind};
use veac_plan::ResolvedInputKind;

use super::support::{
    bindings, file_identity, output_bindings, resolved, test_font_path, text_fixture,
};

#[test]
fn bundle_resources_are_exact_sorted_and_identity_bound() {
    let plan = resolved(&text_fixture(true));
    let bindings = bindings(&plan);
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), plan.inputs.len());
    assert!(bundle
        .protected_resources()
        .windows(2)
        .all(|pair| pair[0].path < pair[1].path));
    for resource in bundle.protected_resources() {
        let input = plan
            .inputs
            .iter()
            .find(|input| {
                bindings
                    .input(&input.id)
                    .and_then(|binding| binding.resource())
                    .is_some_and(|binding| binding.path() == resource.path)
            })
            .unwrap();
        assert_eq!(resource.expected_identity, input.observed_identity);
    }
}

#[test]
fn duplicate_resource_path_deduplicates_only_an_identical_identity() {
    let mut plan = resolved(&text_fixture(true));
    let expected = file_identity(&test_font_path());
    for input in &mut plan.inputs {
        input.observed_identity = expected.clone();
    }
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        bindings.bind_original(input, test_font_path()).unwrap();
    }
    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.protected_resources().len(), 1);
    assert_eq!(bundle.protected_resources()[0].expected_identity, expected);
}

#[test]
fn conflicting_identity_for_one_resource_path_fails_closed() {
    let plan = resolved(&text_fixture(true));
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        bindings.bind_original(input, test_font_path()).unwrap();
    }
    let error = emit_all(&plan, &bindings).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].kind,
        CodegenErrorKind::InvalidResourceBinding
    );
    assert_eq!(error.diagnostics()[0].code, "RESOURCE_BINDING_INVALID");
    assert!(error.diagnostics()[0]
        .message
        .contains("conflicting expected identities"));
}

#[test]
fn non_ascii_resource_paths_use_utf8_sort_order() {
    let plan = resolved(&text_fixture(true));
    let temp = tempfile::tempdir().unwrap();
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        let path = if matches!(input.kind, ResolvedInputKind::Font { .. }) {
            let path = temp.path().join("\u{e000}-font.ttf");
            std::fs::copy(test_font_path(), &path).unwrap();
            path
        } else {
            let path = temp.path().join("\u{10000}-media.bin");
            std::fs::write(&path, b"media").unwrap();
            path
        };
        bindings.bind_original(input, path).unwrap();
    }
    let bundle = emit_all(&plan, &bindings).unwrap();
    let actual: Vec<_> = bundle
        .protected_resources()
        .iter()
        .map(|resource| resource.path.to_str().unwrap())
        .collect();
    let mut expected = actual.clone();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}
