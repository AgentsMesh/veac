use std::path::Path;
use std::process;

use crate::pipeline;

/// Compile and render a .veac file to video output.
pub fn cmd_build(file: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

    // Codegen: produce FFmpeg command(s) — supports multi-output
    let plans = veac_codegen::ffmpeg::generate_all(&ir, output);

    for plan in &plans {
        println!(
            "Building: {} -> {}",
            file.display(),
            plan.output_path.display()
        );

        // Execute FFmpeg
        let args = plan.to_args();
        let status = process::Command::new("ffmpeg")
            .args(&args)
            .status()
            .map_err(|e| format!("failed to run ffmpeg: {e}"))?;

        if !status.success() {
            let code = status.code().unwrap_or(1);
            return Err(format!("ffmpeg exited with code {code}").into());
        }

        println!("Done: {}", plan.output_path.display());
    }
    Ok(())
}

fn read_source(file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(format!("file not found: {}", file.display()).into());
    }
    Ok(std::fs::read_to_string(file)?)
}
