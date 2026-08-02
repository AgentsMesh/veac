#![cfg(unix)]

use veac_provider::{canonical_provider_manifest_bytes, canonical_request_bytes};

use super::support::*;
use super::workflow_provider_support::*;

#[test]
fn provider_run_surfaces_bounded_process_failures_without_publishing_a_response() {
    let temp = tempdir().unwrap();
    let fixture = provider_fixture();
    let request = temp.path().join("request.json");
    let response = temp.path().join("response.json");
    let diagnostics = temp.path().join("oversized-diagnostics.bin");
    std::fs::write(&request, canonical_request_bytes(&fixture.request).unwrap()).unwrap();
    std::fs::write(&diagnostics, vec![b'x'; 2 * 1024 * 1024]).unwrap();
    let manifest = shell(&canonical_provider_manifest_bytes(&fixture.manifest).unwrap());
    let provider = executable(
        temp.path(),
        "oversized-provider.sh",
        &format!(
            "#!/bin/sh\nif [ \"$VEAC_PROVIDER_MODE\" = manifest ]; then printf '%s' {manifest}; exit 0; fi\n/bin/cat \"$1\" >&2\n"
        ),
    );

    let output = veac()
        .args([
            "provider-run",
            request.to_str().unwrap(),
            "--program",
            provider.to_str().unwrap(),
            "--store",
            temp.path().join("store").to_str().unwrap(),
            "--response",
            response.to_str().unwrap(),
            "--arg",
            diagnostics.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("error[PROVIDER_RUN_FAILED]"), "{error}");
    assert!(error.contains("provider stderr exceeds the diagnostic byte limit"));
    assert!(!response.exists());
}

fn shell(value: &[u8]) -> String {
    let value = String::from_utf8(value.to_vec()).unwrap();
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
