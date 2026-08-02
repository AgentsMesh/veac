const PROVENANCE: &str = r#"provenance {
  producer "coverage";
  request-sha256 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
  response-sha256 "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
}"#;

fn annotation(kind: &str, id: &str, target: &str, span: &str, payload: &str) -> String {
    let span_terminator = if span.contains('{') { "" } else { ";" };
    format!(
        "annotation {kind} {id} {{ target {target}; span {span}{span_terminator} payload {{ {payload} }} {PROVENANCE} }}"
    )
}

pub(super) fn all_annotations() -> String {
    [
        annotation(
            "marker",
            "a-marker",
            "project",
            "untimed",
            "label \"Opening\"; color #12345678;",
        ),
        annotation(
            "language",
            "b-language",
            "sequence main",
            "untimed",
            "candidate \"en\" { confidence 0.6; } candidate \"fr\" { confidence 0.9; }",
        ),
        annotation(
            "scene-boundary",
            "c-boundary",
            "layer canvas",
            "point { at 1s; }",
            "confidence 0.8; hard-cut false;",
        ),
        annotation(
            "scene",
            "d-scene",
            "item hero",
            "range { at 0s; duration 2s; }",
            "",
        ),
        annotation(
            "beat",
            "e-beat",
            "sequence main",
            "point { at 2s; }",
            "confidence 0.95; bar 7; beat-in-bar 2; tempo-bpm 120.5; meter 4;",
        ),
        annotation(
            "silence",
            "f-silence",
            "item hero",
            "range { at 2s; duration 500ms; }",
            "mean-db -54db; confidence 0.91;",
        ),
        annotation(
            "filler",
            "g-filler-keep",
            "item hero",
            "range { at 1s; duration 100ms; }",
            "token \"um\"; confidence 0.7; suggestion keep;",
        ),
        annotation(
            "filler",
            "h-filler-delete",
            "item hero",
            "range { at 1.2s; duration 100ms; }",
            "token \"uh\"; confidence 0.8; suggestion delete;",
        ),
        annotation(
            "filler",
            "i-filler-tighten",
            "item hero",
            "range { at 1.4s; duration 100ms; }",
            "token \"like\"; confidence 0.9; suggestion tighten;",
        ),
        annotation(
            "highlight",
            "j-highlight",
            "item hero",
            "range { at 0s; duration 4s; }",
            "score 0.88; rationale \"Strong hook\"; evidence \"z-last\"; evidence \"a-first\";",
        ),
        annotation(
            "review",
            "k-review-keep",
            "item hero",
            "range { at 0s; duration 4s; }",
            "action keep; rationale \"Approved\"; confidence 0.99;",
        ),
        annotation(
            "review",
            "l-review-remove",
            "item hero",
            "range { at 0s; duration 4s; }",
            "action remove; rationale \"Duplicate\"; confidence 0.75;",
        ),
        annotation(
            "marker",
            "m-material",
            "resource poster",
            "untimed",
            "label \"Poster\";",
        ),
    ]
    .join("\n")
}
