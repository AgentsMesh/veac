use std::process::ExitCode;

use super::{error_line, exit_code};

#[test]
fn result_maps_to_process_exit_code_without_duplicate_prefixes() {
    assert_eq!(exit_code(Ok(())), ExitCode::SUCCESS);
    let plain = veac_cli::CliError::rendered("plain");
    let coded = veac_cli::CliError::rendered("error[CODE]: failure");
    assert_eq!(error_line(&plain), "error: plain");
    assert_eq!(error_line(&coded), "error[CODE]: failure");
    assert_eq!(exit_code(Err(plain)), ExitCode::FAILURE);
}

#[test]
fn explicit_dispatch_parses_every_public_command_and_fails_closed() {
    let cases: &[&[&str]] = &[
        &[
            "veac",
            "caption",
            "import",
            "missing",
            "--format",
            "srt",
            "--timescale",
            "24",
            "--overlap",
            "allow",
            "--namespace",
            "ns",
            "--output",
            "out",
            "--loss-report",
            "loss",
            "--allow-lossy",
        ],
        &[
            "veac",
            "caption",
            "export",
            "missing",
            "--format",
            "web-vtt",
            "--output",
            "out",
            "--loss-report",
            "loss",
            "--allow-lossy",
        ],
        &[
            "veac",
            "caption",
            "propose",
            "missing",
            "document",
            "bindings",
            "--operation-id",
            "op_test",
            "--output",
            "out",
        ],
        &[
            "veac", "caption", "extract", "missing", "--track", "trk_test", "bindings", "--output",
            "out",
        ],
        &[
            "veac",
            "otio",
            "export",
            "missing",
            "--sequence",
            "seq_test",
            "--output",
            "out",
            "--loss-report",
            "loss",
            "--allow-lossy",
        ],
        &[
            "veac",
            "otio",
            "propose",
            "missing",
            "timeline",
            "--bindings",
            "bindings",
            "--operation-id",
            "op_test",
            "--output",
            "out",
            "--loss-report",
            "loss",
            "--allow-lossy",
        ],
        &[
            "veac", "derive", "missing", "spec", "--store", "store", "--ffmpeg", "ffmpeg",
        ],
        &[
            "veac",
            "provider-run",
            "missing",
            "--program",
            "program",
            "--store",
            "store",
            "--arg=-x",
            "--response",
            "response",
        ],
        &[
            "veac",
            "provider-propose",
            "missing",
            "request",
            "response",
            "context",
            "--output",
            "out",
        ],
        &[
            "veac",
            "build",
            "missing",
            "--emit-ir",
            "out",
            "--revision",
            "7",
        ],
        &["veac", "check", "missing", "--revision", "7"],
        &["veac", "fmt", "missing", "--stdout"],
        &["veac", "check-ir", "missing"],
        &[
            "veac",
            "edit",
            "missing",
            "batch",
            "--output",
            "out",
            "--dry-run",
        ],
        &[
            "veac", "plan", "missing", "--config", "out_main", "--format", "json",
        ],
        &[
            "veac", "manifest", "missing", "--config", "out_main", "--output", "out",
        ],
        &[
            "veac",
            "bundle",
            "missing",
            "--config",
            "out_main",
            "--destination",
            "bundle",
        ],
        &[
            "veac",
            "render",
            "missing",
            "--config",
            "out_main",
            "--destination",
            "outputs",
        ],
        &["veac", "probe", "missing"],
    ];
    for arguments in cases {
        assert!(
            veac_cli::run_with_args(*arguments).is_err(),
            "{arguments:?}"
        );
    }
    assert!(veac_cli::run_with_args([
        "veac",
        "schema",
        "--contract",
        "provider-edit-proposal",
        "--format",
        "json-schema",
    ])
    .is_ok());
    assert!(veac_cli::run_with_args(["veac", "removed-command"]).is_err());
}
