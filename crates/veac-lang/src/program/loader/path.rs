use std::path::{Component, Path, PathBuf};

pub(super) fn confined_request(requested: &str) -> Result<PathBuf, String> {
    let path = Path::new(requested);
    let valid = !requested.is_empty()
        && requested.len() <= 4096
        && !requested.contains('\0')
        && !requested.contains('\\')
        && !requested.contains(':')
        && !requested.chars().any(char::is_control)
        && !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
        && path
            .components()
            .any(|part| matches!(part, Component::Normal(_)));
    if !valid {
        return Err("module import must be a root-confined relative path".to_owned());
    }
    Ok(path.to_path_buf())
}

pub(crate) fn normalize(path: &Path) -> Result<String, String> {
    let id = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(
                value
                    .to_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "source path is not valid UTF-8".to_owned()),
            ),
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))?;
    validate_source_id(&id)?;
    Ok(id)
}

pub(crate) fn validate_source_id(id: &str) -> Result<(), String> {
    crate::source_edit::validate_module_path(id).map_err(|_| {
        "source loader returned an ID that is not a canonical root-relative source path".to_owned()
    })
}
