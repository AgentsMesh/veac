use std::path::Path;
use std::process;

use crate::pipeline;

/// Show the compilation plan (FFmpeg commands) without executing.
pub fn cmd_plan(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = read_source(file)?;
    let mut ir = match pipeline::compile(&source, file) {
        Ok(ir) => ir,
        Err(e) => {
            eprint!("{e}");
            process::exit(1);
        }
    };

    // Resolve media metadata (duration, has_audio, etc.) via ffprobe with caching.
    let warnings = pipeline::resolve_media(&mut ir, file);
    for w in &warnings {
        eprintln!("warning: {w}");
    }

    // Generate the plan(s) but don't execute — supports multi-output
    let output = Path::new("output.mp4");
    let plans = veac_codegen::ffmpeg::generate_all(&ir, output);
    println!("Compilation plan for: {}\n", file.display());
    for (i, plan) in plans.iter().enumerate() {
        if plans.len() > 1 {
            println!("--- Output {} ({}) ---", i + 1, plan.output_path.display());
        }
        println!("{}", plan.to_command_string());
        if plans.len() > 1 {
            println!();
        }
    }

    Ok(())
}

fn read_source(file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(format!("file not found: {}", file.display()).into());
    }
    Ok(std::fs::read_to_string(file)?)
}
