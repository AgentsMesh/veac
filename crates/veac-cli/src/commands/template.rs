use crate::arguments::TemplateCommand;
use crate::environment::Environment;
use crate::CliResult;

mod replacement;

pub(crate) fn run(command: TemplateCommand, environment: &dyn Environment) -> CliResult {
    match command {
        TemplateCommand::Propose {
            project,
            request,
            output,
        } => propose(project, request, output, environment),
    }
}

fn propose(
    project: std::path::PathBuf,
    request: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    environment: &dyn Environment,
) -> CliResult {
    let loaded = crate::canonical::load_local(&project)?;
    let request_file = crate::fs::canonical_file(&request, "template fill request")?;
    let request: veac_template::TemplateFillRequest =
        super::workflow_io::read_json(&request_file, "template fill request")?;
    let batch = super::workflow_io::result(
        veac_template::propose_template_fill(&loaded.envelope, &request),
        "TEMPLATE_PROPOSAL_FAILED",
    )?;
    let authored_replacements = replacement_material_paths(&request, &loaded.project_file);
    let replacements = replacement::verify(&request, &loaded.project_file, environment)?;
    let bytes = super::workflow_io::result(
        veac_ir::canonical_edit_batch_json(&batch),
        "TEMPLATE_PROPOSAL_FAILED",
    )?;
    let mut protected = vec![loaded.project_file.clone(), request_file];
    protected.extend(crate::canonical::local_material_paths(
        &loaded.envelope,
        &loaded.project_file,
    ));
    protected.extend(authored_replacements);
    protected.extend(replacements);
    super::workflow_io::write(bytes.as_bytes(), output.as_deref(), &protected)
}

pub(crate) fn replacement_material_paths(
    request: &veac_template::TemplateFillRequest,
    project_file: &std::path::Path,
) -> Vec<std::path::PathBuf> {
    let base = project_file.parent().unwrap_or(std::path::Path::new("."));
    request
        .media_bindings
        .iter()
        .filter_map(|binding| match &binding.material.source {
            veac_ir::MaterialSource::File { uri } => Some(base.join(uri)),
            veac_ir::MaterialSource::Remote { .. } => None,
        })
        .collect()
}
