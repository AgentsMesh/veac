use veac_lang::program::expression::Value;

use crate::{
    DeliveryId, DeliveryPathTemplate, MediaType, OutputId, ProjectDelivery, ProjectOutput,
};

use super::super::ProjectDecodeError;
use super::value::{unknown_variant, Decoder};

pub(super) fn output(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectOutput, ProjectDecodeError> {
    let variant = decoder.variant(value, "ProjectOutput", path)?;
    let id = || {
        decoder
            .identifier(variant.fields.get("id")?, &variant.fields.path("id"))
            .map(OutputId::new)
    };
    match variant.name {
        "Media" => Ok(ProjectOutput::Media {
            id: id()?,
            media_type: media_type(
                decoder,
                variant.fields.get("media_type")?,
                &variant.fields.path("media_type"),
            )?,
        }),
        "Data" => Ok(ProjectOutput::Data {
            id: id()?,
            schema: decoder.optional_text(
                variant.fields.get("schema")?,
                &variant.fields.path("schema"),
            )?,
        }),
        "Directory" => Ok(ProjectOutput::Directory { id: id()? }),
        name => Err(unknown_variant(path, "ProjectOutput", name)),
    }
}

fn media_type(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<MediaType, ProjectDecodeError> {
    let variant = decoder.variant(value, "MediaType", path)?;
    match variant.name {
        "Video" => Ok(MediaType::Video),
        "Audio" => Ok(MediaType::Audio),
        "Image" => Ok(MediaType::Image),
        name => Err(unknown_variant(path, "MediaType", name)),
    }
}

pub(super) fn delivery(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectDelivery, ProjectDecodeError> {
    let variant = decoder.variant(value, "ProjectDelivery", path)?;
    let id =
        DeliveryId::new(decoder.identifier(variant.fields.get("id")?, &variant.fields.path("id"))?);
    let output = OutputId::new(decoder.identifier(
        variant.fields.get("output")?,
        &variant.fields.path("output"),
    )?);
    let destination = DeliveryPathTemplate::new(decoder.text(
        variant.fields.get("destination")?,
        &variant.fields.path("destination"),
    )?);
    match variant.name {
        "File" => Ok(ProjectDelivery::File {
            id,
            output,
            destination,
        }),
        "Directory" => Ok(ProjectDelivery::Directory {
            id,
            output,
            destination,
        }),
        name => Err(unknown_variant(path, "ProjectDelivery", name)),
    }
}
