use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::error::{CliError, CliResult};
use veac_lang::program::expression::{PrimitiveType, ValueTypeKind};
use veac_lang::program::{
    BuildInputBinding, BuildInputDeclaration, BuildInputManifestV1, BuildInputManifestValue,
    ExecutableBuild,
};

pub(crate) fn build_inputs(
    path: Option<&Path>,
    inline: &[String],
    program: &ExecutableBuild,
) -> CliResult<BuildInputManifestV1> {
    let mut manifest = read_manifest(path)?;
    reject_manifest_duplicates(&manifest)?;
    let mut bindings = manifest
        .inputs
        .into_iter()
        .map(|binding| (binding.name.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut inline_names = BTreeSet::new();
    for assignment in inline {
        let (name, raw) = assignment.split_once('=').ok_or_else(|| {
            CliError::new(
                "PROGRAM_INPUT_INLINE_SYNTAX",
                format!("inline Build input `{assignment}` must use NAME=VALUE"),
            )
        })?;
        if name.is_empty() {
            return Err(CliError::new(
                "PROGRAM_INPUT_INLINE_SYNTAX",
                "inline Build input name must not be empty",
            ));
        }
        if !inline_names.insert(name) {
            return Err(CliError::new(
                "PROGRAM_INPUT_DUPLICATE",
                format!("inline Build input `{name}` is bound more than once"),
            ));
        }
        let declaration = program
            .build_input_declarations()
            .get(name)
            .ok_or_else(|| {
                CliError::new(
                    "PROGRAM_INPUT_UNKNOWN",
                    format!("unknown Build input `{name}`"),
                )
            })?;
        bindings.insert(
            name.to_owned(),
            BuildInputBinding {
                name: name.to_owned(),
                value: decode(declaration, raw)?,
            },
        );
    }
    manifest.inputs = bindings.into_values().collect();
    Ok(manifest)
}

fn read_manifest(path: Option<&Path>) -> CliResult<BuildInputManifestV1> {
    let Some(path) = path else {
        return Ok(BuildInputManifestV1::empty());
    };
    let path = crate::fs::canonical_file(path, "Build input manifest")?;
    let json = crate::fs::read_utf8_bounded(
        &path,
        "Build input manifest",
        veac_lang::program::MAX_BUILD_INPUT_MANIFEST_BYTES as u64,
    )?;
    veac_lang::program::parse_build_input_manifest(&json)
        .map_err(|error| CliError::new(error.code(), error.message()))
}

fn reject_manifest_duplicates(manifest: &BuildInputManifestV1) -> CliResult {
    let mut names = BTreeSet::new();
    if let Some(binding) = manifest
        .inputs
        .iter()
        .find(|binding| !names.insert(binding.name.as_str()))
    {
        return Err(CliError::new(
            "PROGRAM_INPUT_DUPLICATE",
            format!("Build input `{}` is bound more than once", binding.name),
        ));
    }
    Ok(())
}

fn decode(declaration: &BuildInputDeclaration, raw: &str) -> CliResult<BuildInputManifestValue> {
    let value = match declaration.value_type().kind() {
        ValueTypeKind::Primitive(PrimitiveType::Boolean) => BuildInputManifestValue::Bool {
            value: parse_bool(raw, declaration.name())?,
        },
        ValueTypeKind::Primitive(PrimitiveType::Integer) => BuildInputManifestValue::Integer {
            value: raw.parse().map_err(|_| value_error(declaration, raw))?,
        },
        ValueTypeKind::Primitive(PrimitiveType::Scalar) => BuildInputManifestValue::Scalar {
            value: raw.to_owned(),
        },
        ValueTypeKind::Primitive(PrimitiveType::Text) => BuildInputManifestValue::Text {
            value: raw.to_owned(),
        },
        ValueTypeKind::Primitive(PrimitiveType::Time) => BuildInputManifestValue::Time {
            value: raw.to_owned(),
        },
        ValueTypeKind::Primitive(PrimitiveType::Length) => BuildInputManifestValue::Length {
            value: raw.to_owned(),
        },
        ValueTypeKind::Primitive(PrimitiveType::Angle) => BuildInputManifestValue::Angle {
            value: raw.to_owned(),
        },
        ValueTypeKind::Primitive(PrimitiveType::Color) => BuildInputManifestValue::Color {
            value: raw.to_owned(),
        },
        ValueTypeKind::Nominal(_) => BuildInputManifestValue::Enum {
            value: raw.to_owned(),
        },
        _ => return Err(type_error(declaration)),
    };
    Ok(value)
}

fn parse_bool(raw: &str, name: &str) -> CliResult<bool> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CliError::new(
            "PROGRAM_INPUT_VALUE",
            format!("invalid bool literal `{raw}` for Build input `{name}`"),
        )),
    }
}

fn value_error(declaration: &BuildInputDeclaration, raw: &str) -> CliError {
    CliError::new(
        "PROGRAM_INPUT_VALUE",
        format!(
            "invalid {} literal `{raw}` for Build input `{}`",
            declaration.value_type(),
            declaration.name()
        ),
    )
}

fn type_error(declaration: &BuildInputDeclaration) -> CliError {
    CliError::new(
        "PROGRAM_INPUT_TYPE_MISMATCH",
        format!(
            "Build input `{}` has unsupported inline type {}",
            declaration.name(),
            declaration.value_type()
        ),
    )
}
