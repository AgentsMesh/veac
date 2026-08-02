mod budget;
mod component;
pub(crate) mod definition;
pub(crate) mod hygiene;
mod inject;
mod origin;
mod parameters;
mod preset;
mod scope;
mod slot;
mod slot_media;
mod text;

use super::diagnostic::Diagnostic;
use super::model::{ComponentCatalog, Scope, SurfaceFile};
use super::provenance::ProvenanceMap;

pub(crate) fn project(
    file: &SurfaceFile,
    scope: &Scope,
    catalog: &ComponentCatalog,
) -> Result<(String, ProvenanceMap), Diagnostic> {
    let (source, provenance, _) = project_in(file, scope::View::from(scope), catalog)?;
    Ok((source, provenance))
}

fn project_in(
    file: &SurfaceFile,
    scope: scope::View<'_>,
    catalog: &ComponentCatalog,
) -> Result<(String, ProvenanceMap, usize), Diagnostic> {
    let project = file
        .project
        .as_ref()
        .expect("entry parser requires a project");
    let raw = &file.source[project.span.start..project.span.end];
    let expanded = text::expand(&file.path, raw, scope.presets, scope.values, 0)?;
    let mut sequences = inject::Sequences::new(&file.path, expanded)?;
    let mut provenance = ProvenanceMap::default();
    let mut registry = hygiene::Registry::default();
    let mut state = component::State::default();
    registry.source(&file.path, sequences.project())?;
    for instance in &file.instances {
        registry.explicit(&file.path, &instance.id, instance.span)?;
    }
    for instance in &file.instances {
        component::expand(
            file,
            scope,
            catalog,
            instance,
            &mut provenance,
            &mut registry,
            &mut sequences,
            &mut state,
        )?;
    }
    Ok((sequences.finish(), provenance, state.nodes()))
}

pub(crate) fn preset_body(path: &str, source: &str, scope: &Scope) -> Result<String, Diagnostic> {
    text::expand(
        path,
        source,
        scope.presets.as_ref(),
        scope.values.as_ref(),
        0,
    )
}

pub(crate) use preset::{uses as preset_uses, PresetUse};
pub(crate) use slot_media::validate as validate_slot_media;
