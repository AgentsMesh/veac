mod commands;
mod pipeline;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

/// VEAC - Video Editing as Code compiler.
#[derive(Parser)]
#[command(name = "veac", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile and render video from a .veac source file.
    Build {
        /// Path to the .veac source file.
        file: PathBuf,
        /// Output file path (e.g. output.mp4).
        #[arg(short, long, default_value = "output.mp4")]
        output: PathBuf,
    },

    /// Validate syntax and semantics without rendering.
    Check {
        /// Path to the .veac source file.
        file: PathBuf,
    },

    /// Show the compilation plan (dry-run, prints FFmpeg commands).
    Plan {
        /// Path to the .veac source file.
        file: PathBuf,
    },

    /// Format a .veac source file (pretty-print).
    Fmt {
        /// Path to the .veac source file.
        file: PathBuf,
    },

    /// Probe a media file and display its info.
    Probe {
        /// Path to the media file.
        file: PathBuf,
    },

    /// Batch render from a template with CSV variable overrides.
    Batch {
        /// Path to the .veac template file.
        file: PathBuf,
        /// Path to a CSV file with variable overrides per row.
        #[arg(long)]
        params: PathBuf,
        /// Output directory for rendered videos.
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Print the complete VEAC language syntax reference.
    Syntax,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Build { file, output } => commands::cmd_build(&file, &output),
        Command::Check { file } => commands::cmd_check(&file),
        Command::Plan { file } => commands::cmd_plan(&file),
        Command::Fmt { file } => commands::cmd_fmt(&file),
        Command::Probe { file } => commands::cmd_probe(&file),
        Command::Batch {
            file,
            params,
            output,
        } => commands::cmd_batch(&file, &params, &output),
        Command::Syntax => commands::cmd_syntax(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
