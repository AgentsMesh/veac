mod arguments;
mod canonical;
mod commands;
mod diagnostic;
mod environment;
mod error;
mod frontend;
mod fs;
mod material_root;
mod output;
mod planning;

use std::ffi::OsString;

use arguments::{Cli, Command, PlanFormat, SchemaContract, SchemaFormat};

pub use diagnostic::{CliDiagnostic, DiagnosticFormat, SourceSpan};
pub use error::{CliError, CliResult};

pub fn run() -> CliResult {
    execute(Cli::parse())
}

/// Parse and execute one explicit CLI invocation without mutating process arguments.
pub fn run_with_args<I, T>(arguments: I) -> CliResult
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli =
        Cli::try_parse_from(arguments).map_err(|error| CliError::rendered(error.to_string()))?;
    execute(cli)
}

fn execute(cli: Cli) -> CliResult {
    let environment = environment::SystemEnvironment::default();
    execute_with_environment(cli, &environment)
}

fn execute_with_environment(cli: Cli, environment: &dyn environment::Environment) -> CliResult {
    let format = cli.diagnostic_format;
    let result = match cli.command {
        Command::Project { command } => commands::project(command),
        Command::Caption { command } => commands::caption(command),
        Command::Otio { command } => commands::otio(command),
        Command::Template { command } => commands::template(command, environment),
        Command::Derive(arguments) => commands::derive(arguments),
        Command::IngestAnalysis(arguments) => commands::ingest_analysis(arguments),
        Command::ProviderRun(arguments) => commands::provider_run(arguments),
        Command::ProviderPropose(arguments) => commands::provider_propose(arguments),
        Command::Artifact { command } => commands::artifact(command),
        Command::PackageBindings(arguments) => commands::package_bindings(arguments),
        Command::Relink(arguments) => commands::relink(arguments),
        Command::Build(arguments::BuildSourceArgs {
            source,
            emit_ir,
            inputs,
            inline_inputs,
            material_root,
            package_roots,
            revision,
        }) => commands::build(
            &source,
            emit_ir.as_deref(),
            inputs.as_deref(),
            &inline_inputs,
            material_root.as_deref(),
            &package_roots,
            revision,
        ),
        Command::Check(arguments::CheckSourceArgs {
            source,
            inputs,
            inline_inputs,
            package_roots,
            revision,
        }) => commands::check(
            &source,
            inputs.as_deref(),
            &inline_inputs,
            &package_roots,
            revision,
        ),
        Command::Fmt(arguments::FormatSourceArgs {
            source,
            check,
            stdout,
            package_roots,
        }) => commands::format(&source, check, stdout, &package_roots),
        Command::CheckIr { project } => commands::check_ir(&project),
        Command::Edit {
            project,
            edit_batch,
            output,
            dry_run,
        } => commands::edit(&project, &edit_batch, output.as_deref(), dry_run),
        Command::SourceRevision(arguments::SourceGraphArgs {
            source,
            package_roots,
        }) => commands::source_revision(&source, &package_roots),
        Command::SourceIndex(arguments::SourceGraphArgs {
            source,
            package_roots,
        }) => commands::source_index(&source, &package_roots),
        Command::SourceEdit(arguments::SourceEditArgs {
            source,
            source_edit_batch,
            inputs,
            inline_inputs,
            package_roots,
            output,
            dry_run,
        }) => commands::source_edit(
            &source,
            &source_edit_batch,
            inputs.as_deref(),
            &inline_inputs,
            &package_roots,
            output.as_deref(),
            dry_run,
        ),
        Command::Schema { contract, format } => commands::schema(contract, format),
        Command::LanguageSpec { schema } => commands::language_spec(schema),
        Command::LanguagePackage { command } => commands::language_package(command),
        Command::Plan {
            project,
            config,
            bindings,
            material_root,
            format,
        } => commands::plan(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            material_root.as_deref(),
            format,
            environment,
        ),
        Command::Manifest {
            project,
            config,
            bindings,
            material_root,
            output,
        } => commands::manifest(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            material_root.as_deref(),
            output.as_deref(),
            environment,
        ),
        Command::Bundle {
            project,
            config,
            bindings,
            material_root,
            destination,
        } => commands::bundle(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            material_root.as_deref(),
            &destination,
            environment,
        ),
        Command::Render {
            project,
            config,
            bindings,
            material_root,
            destination,
            proxy_policy,
            render_segment_policy,
        } => commands::render(
            &project,
            config.as_deref(),
            planning::InputResolution::new(material_root.as_deref(), bindings.as_deref()),
            destination.as_deref(),
            proxy_policy,
            render_segment_policy,
            environment,
        ),
        Command::Probe {
            input,
            material,
            material_root,
        } => commands::probe(
            &input,
            material.as_deref(),
            material_root.as_deref(),
            environment,
        ),
    };
    result.map_err(|error| error.with_diagnostic_format(format))
}

#[cfg(test)]
mod unit_tests;
