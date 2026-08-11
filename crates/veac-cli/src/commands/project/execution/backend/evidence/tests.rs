use super::cancelled;
use veac_build::CancellationToken;

#[test]
fn cancellation_preserves_the_evidence_phase() {
    let token = CancellationToken::new();
    assert!(cancelled(&token, "before preparation").is_ok());

    token.cancel();
    let error = cancelled(&token, "after observation").unwrap_err();
    assert!(error.to_string().contains("after observation"));
}
