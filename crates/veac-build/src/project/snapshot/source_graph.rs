use std::path::Path;

use veac_project::ProjectPath;

use crate::{BuildError, BuildResult, ProjectFileSnapshot, ProjectSourceGraphRevision};

pub(super) fn capture(
    root: &Path,
    path: &ProjectPath,
    packages: &super::ProjectPackageSet,
) -> BuildResult<(ProjectFileSnapshot, ProjectSourceGraphRevision)> {
    let entry_path = super::checked_file(root, path)?;
    packages.revalidate()?;
    let (project, entry_source) = veac_lang::program::FileSystemLoader::for_entry(&entry_path)
        .map_err(|error| {
            BuildError::invalid(format!("cannot load project source graph: {error}"))
        })?;
    let loader = packages.loader(project)?;
    let prepared =
        veac_lang::program::prepare_with_loader(entry_source, &loader).map_err(|error| {
            BuildError::invalid(format!(
                "project VEAC source graph {} is invalid: {error}",
                path.as_str()
            ))
        })?;
    packages.revalidate()?;
    let root_module = prepared.root_module();
    let Some(entry_name) = entry_path.file_name() else {
        return Err(BuildError::invalid("project VEAC entry has no file name"));
    };
    let Some(expected_root) = entry_name.to_str() else {
        return Err(BuildError::invalid(
            "project VEAC entry name is not valid UTF-8",
        ));
    };
    check_root_module(root_module, expected_root)?;
    let source = super::graph_root_source(prepared.sources(), root_module, "project VEAC")?;
    let entry = ProjectFileSnapshot {
        path: path.as_str().to_owned(),
        content: veac_artifact::ContentDigest::sha256(source.as_bytes()),
        size_bytes: source.len() as u64,
    };
    let authored_sources = prepared.source_graph().project_sources();
    let modules = authored_sources
        .iter()
        .map(|(module, source)| veac_lang::source_edit::SourceModule::utf8(module, source))
        .collect::<Vec<_>>();
    let revision = match veac_lang::source_edit::source_graph_revision(&modules) {
        Ok(revision) => revision,
        Err(error) => {
            return Err(BuildError::invalid(format!(
                "project VEAC revision failed: {error}"
            )))
        }
    };
    let authored_module_count = super::graph_module_count(modules.len(), "project VEAC")?;
    let authored_modules = authored_sources.keys().cloned().collect();
    Ok((
        entry,
        ProjectSourceGraphRevision {
            root_module: root_module.to_owned(),
            authored_source_graph_sha256: revision.source_graph_sha256,
            complete_source_graph_sha256: prepared
                .source_graph()
                .complete_revision()
                .sha256()
                .to_owned(),
            authored_module_count,
            authored_modules,
        },
    ))
}

fn check_root_module(actual: &str, expected: &str) -> BuildResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(BuildError::invalid(
            "project VEAC source graph returned an unexpected root module",
        ))
    }
}

#[cfg(test)]
#[path = "source_graph/coverage_tests.rs"]
mod coverage_tests;
