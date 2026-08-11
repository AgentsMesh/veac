use veac_lang::program::expression::Value;

use crate::{AxisId, FactId, InputId, OutputId, ProjectInput, ProjectInputSource, ProjectPath};

use super::super::ProjectDecodeError;
use super::literal::literal;
use super::selector::target_ref;
use super::value::{unknown_variant, Decoder};

pub(super) fn input(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectInput, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectInput", path)?;
    Ok(ProjectInput {
        id: InputId::new(decoder.identifier(fields.get("id")?, &fields.path("id"))?),
        source: source(decoder, fields.get("source")?, &fields.path("source"))?,
    })
}

fn source(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectInputSource, ProjectDecodeError> {
    let variant = decoder.variant(value, "ProjectInputSource", path)?;
    let get = |name| variant.fields.get(name);
    let at = |name| variant.fields.path(name);
    let path_value = |name| decoder.text(get(name)?, &at(name)).map(ProjectPath::new);
    let output = || {
        decoder
            .identifier(get("output")?, &at("output"))
            .map(OutputId::new)
    };
    let fact = || {
        decoder
            .identifier(get("fact")?, &at("fact"))
            .map(FactId::new)
    };
    let reference = || target_ref(decoder, get("target")?, &at("target"));
    match variant.name {
        "Literal" => Ok(ProjectInputSource::Literal {
            value: literal(decoder, get("value")?, &at("value"))?,
        }),
        "ProfileBinding" => Ok(ProjectInputSource::ProfileBinding {}),
        "LocaleBinding" => Ok(ProjectInputSource::LocaleBinding {}),
        "MatrixBinding" => Ok(ProjectInputSource::MatrixBinding {
            axis: AxisId::new(decoder.identifier(get("axis")?, &at("axis"))?),
        }),
        "ProjectMaterial" => Ok(ProjectInputSource::ProjectMaterial {
            path: path_value("path")?,
        }),
        "Artifact" => Ok(ProjectInputSource::Artifact {
            target: reference()?,
            output: output()?,
        }),
        "AssetFact" => Ok(ProjectInputSource::AssetFact {
            path: path_value("path")?,
            fact: fact()?,
        }),
        "AnalysisFact" => Ok(ProjectInputSource::AnalysisFact {
            target: reference()?,
            output: output()?,
            fact: fact()?,
        }),
        name => Err(unknown_variant(path, "ProjectInputSource", name)),
    }
}
