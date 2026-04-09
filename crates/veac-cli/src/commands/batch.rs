/// Batch render: apply CSV variable overrides to a template and render each row.
use std::path::Path;
use std::process;

use crate::pipeline;

pub fn cmd_batch(
    file: &Path,
    params: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(format!("template file not found: {}", file.display()).into());
    }
    if !params.exists() {
        return Err(format!("params CSV not found: {}", params.display()).into());
    }

    // Ensure output directory exists.
    std::fs::create_dir_all(output_dir)?;

    let template_source = std::fs::read_to_string(file)?;

    // Read CSV: first row is headers (variable names), subsequent rows are values.
    let mut reader = csv::Reader::from_path(params)?;
    let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_string()).collect();

    let records: Vec<csv::StringRecord> = reader.records().collect::<Result<_, _>>()?;
    let total = records.len();

    if total == 0 {
        println!("No rows in CSV, nothing to render.");
        return Ok(());
    }

    println!("Batch rendering {total} variants from {}", file.display());

    for (i, record) in records.iter().enumerate() {
        let row_num = i + 1;

        // Build variable override lines: `let var_name = "value"`
        let mut overrides = String::new();
        for (col, header) in headers.iter().enumerate() {
            if let Some(val) = record.get(col) {
                // Inject as let declaration before the template source.
                overrides.push_str(&format!("let {header} = \"{val}\"\n"));
            }
        }

        // Prepend overrides to template source.
        let full_source = format!("{overrides}{template_source}");

        // Determine output filename.
        let output_path = output_dir.join(format!("row_{row_num}.mp4"));

        // Compile
        let mut ir = match pipeline::compile(&full_source, file) {
            Ok(ir) => ir,
            Err(e) => {
                eprintln!("[{row_num}/{total}] Compilation error:\n{e}");
                continue;
            }
        };

        // Resolve media metadata (duration, has_audio, etc.) via ffprobe with caching.
        let warnings = pipeline::resolve_media(&mut ir, file);
        for w in &warnings {
            eprintln!("warning: {w}");
        }

        // Generate FFmpeg command(s) — supports multi-output
        let plans = veac_codegen::ffmpeg::generate_all(&ir, &output_path);

        // Execute each output
        for plan in &plans {
            print!(
                "[{row_num}/{total}] Rendering {}...",
                plan.output_path.display()
            );
            let args = plan.to_args();
            let status = process::Command::new("ffmpeg")
                .args(&args)
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .status()
                .map_err(|e| format!("failed to run ffmpeg: {e}"))?;

            if status.success() {
                println!(" done");
            } else {
                let code = status.code().unwrap_or(1);
                println!(" FAILED (exit code {code})");
            }
        }
    }

    println!("Batch complete: {total} variants processed.");
    Ok(())
}
