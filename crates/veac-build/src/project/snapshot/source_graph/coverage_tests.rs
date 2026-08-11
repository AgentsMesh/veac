use super::check_root_module;

#[test]
fn source_graph_root_identity_is_exact() {
    check_root_module("main.veac", "main.veac").unwrap();
    let error = check_root_module("other.veac", "main.veac").unwrap_err();
    assert!(error.message().contains("unexpected root module"));
}
