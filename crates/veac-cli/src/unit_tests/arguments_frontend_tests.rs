use crate::arguments::Cli;

#[test]
fn legacy_frontend_selection_and_compile_command_are_not_public() {
    for command in ["check", "fmt", "source-revision", "source-index"] {
        assert!(
            Cli::try_parse_from(["veac", command, "main.veac", "--frontend", "executable",])
                .is_err()
        );
    }
    assert!(Cli::try_parse_from(["veac", "compile", "main.veac"]).is_err());
}
