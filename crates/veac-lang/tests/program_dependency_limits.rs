use veac_lang::program::prepare_source;

#[path = "program_functions/support.rs"]
mod support;

#[test]
fn constant_dependency_depth_fails_deterministically() {
    let declarations = chain(65, |name, next| format!("const scalar {name} = {next};"));
    assert_limit(
        &entry(&declarations),
        "constant",
        "PROGRAM_DEPENDENCY_DEPTH",
    );
}

#[test]
fn wide_dependency_graph_fails_at_the_node_budget() {
    let declarations = (0..=16_384)
        .map(|index| format!("const scalar n{index:05} = 1.0;"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_limit(
        &entry(&declarations),
        "constant",
        "PROGRAM_DEPENDENCY_BUDGET",
    );
}

fn chain(count: usize, declaration: impl Fn(&str, &str) -> String) -> String {
    (0..count)
        .map(|index| {
            let name = format!("n{index:03}");
            let next = if index + 1 == count {
                "1.0".to_owned()
            } else {
                format!("n{:03}", index + 1)
            };
            declaration(&name, &next)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_limit(source: &str, dependency: &str, code: &str) {
    let error = prepare_source(source).unwrap_err();
    let diagnostic = &error.as_slice()[0];
    assert_eq!(diagnostic.code, code);
    assert!(diagnostic.message.contains(dependency));
}

fn entry(declarations: &str) -> String {
    support::project_with(declarations, "1s")
}
