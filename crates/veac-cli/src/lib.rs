mod arguments;
mod canonical;
mod commands;
mod diagnostic;
mod environment;
mod error;
mod frontend;
mod fs;
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
        Command::Caption { command } => commands::caption(command),
        Command::Otio { command } => commands::otio(command),
        Command::Template { command } => commands::template(command, environment),
        Command::Derive(arguments) => commands::derive(arguments),
        Command::ProviderRun(arguments) => commands::provider_run(arguments),
        Command::ProviderPropose(arguments) => commands::provider_propose(arguments),
        Command::Artifact { command } => commands::artifact(command),
        Command::PackageBindings(arguments) => commands::package_bindings(arguments),
        Command::Relink(arguments) => commands::relink(arguments),
        Command::Compile {
            source,
            emit_ir,
            revision,
        } => commands::compile(&source, emit_ir.as_deref(), revision),
        Command::Check { source, revision } => commands::check(&source, revision),
        Command::Fmt {
            source,
            check,
            stdout,
        } => commands::format(&source, check, stdout),
        Command::CheckIr { project } => commands::check_ir(&project),
        Command::Edit {
            project,
            edit_batch,
            output,
            dry_run,
        } => commands::edit(&project, &edit_batch, output.as_deref(), dry_run),
        Command::Schema { contract, format } => commands::schema(contract, format),
        Command::Plan {
            project,
            config,
            bindings,
            format,
        } => commands::plan(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            format,
            environment,
        ),
        Command::Manifest {
            project,
            config,
            bindings,
            output,
        } => commands::manifest(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            output.as_deref(),
            environment,
        ),
        Command::Package {
            project,
            config,
            bindings,
            destination,
        } => commands::package(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            &destination,
            environment,
        ),
        Command::Render {
            project,
            config,
            bindings,
            destination,
            proxy_policy,
            render_segment_policy,
        } => commands::render(
            &project,
            config.as_deref(),
            bindings.as_deref(),
            destination.as_deref(),
            proxy_policy,
            render_segment_policy,
            environment,
        ),
        Command::Probe { media } => commands::probe(&media, environment),
    };
    result.map_err(|error| error.with_diagnostic_format(format))
}

#[cfg(test)]
mod unit_tests;
