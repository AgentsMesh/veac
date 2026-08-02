use std::process::ExitCode;

fn main() -> ExitCode {
    exit_code(veac_cli::run())
}

fn exit_code(result: veac_cli::CliResult) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.uses_json_diagnostics() {
                eprintln!("{error}");
            } else {
                eprintln!("{}", error_line(&error));
            }
            ExitCode::FAILURE
        }
    }
}

fn error_line(error: &veac_cli::CliError) -> String {
    let message = error.to_string();
    if message.starts_with("error[") {
        message
    } else {
        format!("error: {message}")
    }
}

#[cfg(test)]
#[path = "main/tests.rs"]
mod tests;
