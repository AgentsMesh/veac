use crate::arguments::{
    CaptionCommand, Cli, Command, OtioCommand, PlanFormat, SchemaContract, SchemaFormat,
    TemplateCommand,
};

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args).unwrap()
}

#[test]
fn clap_exposes_only_the_canonical_pipeline_shapes() {
    assert!(matches!(
        parse(&[
            "veac",
            "caption",
            "import",
            "captions.srt",
            "--format",
            "srt"
        ])
        .command,
        Command::Caption {
            command: CaptionCommand::Import { .. }
        }
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "template",
            "propose",
            "project.json",
            "request.json",
            "-o",
            "batch.json"
        ])
        .command,
        Command::Template {
            command: TemplateCommand::Propose {
                output: Some(_),
                ..
            }
        }
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "otio",
            "propose",
            "project.json",
            "timeline.otio",
            "--operation-id",
            "op_import"
        ])
        .command,
        Command::Otio {
            command: OtioCommand::Propose { .. }
        }
    ));
    assert!(matches!(
        parse(&["veac", "compile", "main.veac"]).command,
        Command::Compile { emit_ir: None, .. }
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "derive",
            "input.mp4",
            "spec.json",
            "--store",
            "cache"
        ])
        .command,
        Command::Derive(_)
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "provider-run",
            "request.json",
            "--program",
            "provider",
            "--store",
            "cache",
            "--arg",
            "literal"
        ])
        .command,
        Command::ProviderRun(_)
    ));
    assert!(matches!(
        parse(&[
            "veac",
            "provider-propose",
            "project.json",
            "request.json",
            "response.json",
            "context.json"
        ])
        .command,
        Command::ProviderPropose(_)
    ));
    assert!(matches!(
        parse(&["veac", "check", "main.veac", "--revision", "7"]).command,
        Command::Check { revision: 7, .. }
    ));
    assert!(matches!(
        parse(&["veac", "fmt", "main.veac", "--check"]).command,
        Command::Fmt { check: true, .. }
    ));
    assert!(matches!(
        parse(&["veac", "check-ir", "project.json"]).command,
        Command::CheckIr { .. }
    ));
    assert!(matches!(
        parse(&["veac", "source-index", "main.veac"]).command,
        Command::SourceIndex { .. }
    ));
    assert!(matches!(
        parse(&["veac", "edit", "project.json", "batch.json", "--dry-run"]).command,
        Command::Edit { dry_run: true, .. }
    ));
    assert!(matches!(
        parse(&["veac", "schema", "--format", "json-schema"]).command,
        Command::Schema {
            contract: SchemaContract::Project,
            format: SchemaFormat::JsonSchema
        }
    ));
    assert!(matches!(
        parse(&["veac", "schema", "--contract", "template-fill-request"]).command,
        Command::Schema {
            contract: SchemaContract::TemplateFillRequest,
            ..
        }
    ));
    assert!(matches!(
        parse(&["veac", "plan", "project.json", "--format", "json"]).command,
        Command::Plan {
            format: PlanFormat::Json,
            ..
        }
    ));
    assert!(matches!(
        parse(&["veac", "render", "project.json", "--destination", "outputs"]).command,
        Command::Render {
            destination: Some(_),
            ..
        }
    ));
    assert!(matches!(
        parse(&["veac", "manifest", "project.json", "-o", "build.json"]).command,
        Command::Manifest {
            output: Some(_),
            ..
        }
    ));
    assert!(matches!(
        parse(&["veac", "package", "project.json", "--destination", "bundle"]).command,
        Command::Package { .. }
    ));
    assert!(matches!(
        parse(&["veac", "probe", "clip.mp4"]).command,
        Command::Probe { .. }
    ));
    assert!(Cli::try_parse_from(["veac", "build", "main.veac"]).is_err());
    assert!(Cli::try_parse_from(["veac", "batch", "main.veac"]).is_err());
    assert!(Cli::try_parse_from(["veac", "fmt", "x", "--check", "--stdout"]).is_err());
    assert!(Cli::try_parse_from([
        "veac",
        "caption",
        "export",
        "captions.json",
        "--format",
        "srt",
        "--allow-lossy"
    ])
    .is_err());
    assert!(
        Cli::try_parse_from(["veac", "otio", "export", "project.json", "--allow-lossy"]).is_err()
    );
}
