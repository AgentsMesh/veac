mod action;
mod derivation;
mod input;
mod literal;
mod output;
mod profile;
mod selector;
mod target;
mod value;

use veac_lang::program::expression::Value;
use veac_lang::program::TypeRegistry;

use crate::{
    LocaleId, ProjectDefaults, ProjectId, ProjectLocale, ProjectManifestV1, ProjectPath,
    ProjectPaths,
};

use self::value::Decoder;
use super::ProjectDecodeError;

pub(super) fn manifest(
    value: &Value,
    types: &TypeRegistry,
) -> Result<ProjectManifestV1, ProjectDecodeError> {
    let decoder = Decoder::new(types);
    let fields = decoder.structure(value, "ProjectManifest", "manifest")?;
    Ok(ProjectManifestV1 {
        schema: decoder.text(fields.get("schema")?, &fields.path("schema"))?,
        version: decoder.u32(fields.get("version")?, &fields.path("version"))?,
        id: ProjectId::new(decoder.identifier(fields.get("id")?, &fields.path("id"))?),
        paths: paths(&decoder, fields.get("paths")?, &fields.path("paths"))?,
        defaults: defaults(&decoder, fields.get("defaults")?, &fields.path("defaults"))?,
        locales: decoder.list_map(fields.get("locales")?, &fields.path("locales"), locale)?,
        profiles: decoder.list_map(
            fields.get("profiles")?,
            &fields.path("profiles"),
            profile::profile,
        )?,
        targets: decoder.list_map(
            fields.get("targets")?,
            &fields.path("targets"),
            target::target,
        )?,
    })
}

fn paths(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectPaths, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectPaths", path)?;
    let path_value = |name| {
        decoder
            .text(fields.get(name)?, &fields.path(name))
            .map(ProjectPath::new)
    };
    Ok(ProjectPaths {
        source_base: path_value("source_base")?,
        material_root: path_value("material_root")?,
        build_root: path_value("build_root")?,
        cache_root: path_value("cache_root")?,
        delivery_root: path_value("delivery_root")?,
    })
}

fn defaults(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectDefaults, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectDefaults", path)?;
    Ok(ProjectDefaults {
        profile: decoder
            .optional_identifier(fields.get("profile")?, &fields.path("profile"))?
            .map(crate::ProfileId::new),
        locale: decoder
            .optional_identifier(fields.get("locale")?, &fields.path("locale"))?
            .map(LocaleId::new),
        max_instances_per_target: decoder.u32(
            fields.get("max_instances_per_target")?,
            &fields.path("max_instances_per_target"),
        )?,
        max_total_instances: decoder.u32(
            fields.get("max_total_instances")?,
            &fields.path("max_total_instances"),
        )?,
    })
}

fn locale(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectLocale, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectLocale", path)?;
    Ok(ProjectLocale {
        id: LocaleId::new(decoder.identifier(fields.get("id")?, &fields.path("id"))?),
        language_tag: decoder.text(fields.get("language_tag")?, &fields.path("language_tag"))?,
    })
}
