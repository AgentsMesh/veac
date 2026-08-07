use super::super::{BindingRoot, CoreValueMetadata, DependencyMask, Stage};

pub(super) fn bind(
    mut stage: Stage,
    dependencies: &DependencyMask,
    parameters: &[CoreValueMetadata],
    captures: &[CoreValueMetadata],
) -> Option<(Stage, DependencyMask)> {
    let mut output = dependencies.without_bindings();
    for dependency in dependencies.shape_bindings() {
        let value =
            value(dependency.root, parameters, captures)?.project_path(&dependency.path, false)?;
        stage = stage.join(value.shape_stage);
        output.union(&value.shape_dependencies);
    }
    for dependency in dependencies.leaf_bindings() {
        let value =
            value(dependency.root, parameters, captures)?.project_path(&dependency.path, false)?;
        stage = stage.join(value.leaf_stage);
        output.union(&value.leaf_dependencies);
    }
    Some((stage, output))
}

fn value<'a>(
    root: BindingRoot,
    parameters: &'a [CoreValueMetadata],
    captures: &'a [CoreValueMetadata],
) -> Option<&'a CoreValueMetadata> {
    match root {
        BindingRoot::Parameter(index) => parameters.get(index),
        BindingRoot::Capture(index) => captures.get(index),
    }
}
