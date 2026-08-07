use std::path::Path;

use crate::error::{CliError, CliResult};

pub(crate) fn run(
    source: &Path,
    emit_ir: Option<&Path>,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    requested_material_root: Option<&Path>,
    revision: u64,
) -> CliResult {
    let (envelope, program, root) =
        crate::frontend::build_graph(source, inputs, inline_inputs, revision)?;
    let material_root =
        crate::material_root::MaterialRoot::for_build(&root, requested_material_root)?;
    let mut json = veac_ir::canonical_json(&envelope)
        .map_err(|error| CliError::new("CANONICAL_ENCODE", error.to_string()))?;
    json.push('\n');
    let Some(destination) = emit_ir else {
        return crate::fs::write_stdout(&json);
    };
    if destination == Path::new("-") {
        return crate::fs::write_stdout(&json);
    }
    let mut protected = program
        .sources()
        .keys()
        .map(|module| root.join(module))
        .collect::<Vec<_>>();
    if let Some(inputs) = inputs {
        protected.push(crate::fs::canonical_file(inputs, "Build input manifest")?);
    }
    protected.push(root.join(crate::fs::SOURCE_LOCK_NAME));
    protected.extend(material_root.local_paths(&envelope));
    let destination = crate::output::guarded_write_many(
        destination,
        protected.iter().map(std::path::PathBuf::as_path),
    )?;
    crate::material_root::require_detached_output_contract(
        &envelope,
        &root,
        &material_root,
        &destination,
    )?;
    crate::fs::atomic_write(&destination, &json)?;
    println!("Canonical IR written: {}", destination.display());
    Ok(())
}
