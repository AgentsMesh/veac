use super::support::*;

const SOURCE: &str = include_str!("../fixtures/text-graphics.veac");

#[test]
fn cli_build_and_format_preserve_agent_authored_text_and_graphics() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, SOURCE);
    let output = temp.path().join("project.json");
    veac()
        .args([
            "build",
            source.to_str().unwrap(),
            "--emit-ir",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    let clips = &json["project"]["sequences"][0]["tracks"][0]["clips"];
    assert_eq!(clips[0]["source"]["generator"]["type"], "gradient");
    assert_eq!(
        clips[1]["source"]["generator"]["shape"]["geometry"]["type"],
        "rounded_rectangle"
    );
    assert_eq!(clips[2]["source"]["style"]["layout"]["wrap"], "word");

    veac()
        .args(["fmt", source.to_str().unwrap()])
        .assert()
        .success();
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
    let formatted = std::fs::read_to_string(source).unwrap();
    assert!(formatted.contains("generator_gradient(gradient_linear("));
    assert!(formatted.contains("during(0s, 1s)"));
}
