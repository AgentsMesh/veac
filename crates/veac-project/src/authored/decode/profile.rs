use veac_lang::program::expression::Value;

use crate::{ExecutionPolicy, ProfileId, ProjectProfile, ProxyPolicy, SegmentationPolicy};

use super::super::ProjectDecodeError;
use super::literal::rational;
use super::value::{unknown_variant, Decoder};

pub(super) fn profile(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectProfile, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectProfile", path)?;
    Ok(ProjectProfile {
        id: ProfileId::new(decoder.identifier(fields.get("id")?, &fields.path("id"))?),
        execution: execution(decoder, fields.get("execution")?, &fields.path("execution"))?,
        proxy: proxy(decoder, fields.get("proxy")?, &fields.path("proxy"))?,
        segmentation: segmentation(
            decoder,
            fields.get("segmentation")?,
            &fields.path("segmentation"),
        )?,
    })
}

fn execution(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ExecutionPolicy, ProjectDecodeError> {
    let variant = decoder.variant(value, "ExecutionPolicy", path)?;
    let get = |name| variant.fields.get(name);
    let at = |name| variant.fields.path(name);
    match variant.name {
        "Serial" => Ok(ExecutionPolicy::Serial {}),
        "Parallel" => Ok(ExecutionPolicy::Parallel {
            max_tasks: decoder.u16(get("max_tasks")?, &at("max_tasks"))?,
        }),
        "ResourceAware" => Ok(ExecutionPolicy::ResourceAware {
            max_tasks: decoder.u16(get("max_tasks")?, &at("max_tasks"))?,
            cpu_threads: decoder.u16(get("cpu_threads")?, &at("cpu_threads"))?,
            memory_mib: decoder.u32(get("memory_mib")?, &at("memory_mib"))?,
            gpu_slots: decoder.u16(get("gpu_slots")?, &at("gpu_slots"))?,
        }),
        name => Err(unknown_variant(path, "ExecutionPolicy", name)),
    }
}

fn proxy(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProxyPolicy, ProjectDecodeError> {
    let variant = decoder.variant(value, "ProxyPolicy", path)?;
    match variant.name {
        "Disabled" => Ok(ProxyPolicy::Disabled {}),
        "PreferExisting" => Ok(ProxyPolicy::PreferExisting {}),
        "Require" => Ok(ProxyPolicy::Require {}),
        name => Err(unknown_variant(path, "ProxyPolicy", name)),
    }
}

fn segmentation(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<SegmentationPolicy, ProjectDecodeError> {
    let variant = decoder.variant(value, "SegmentationPolicy", path)?;
    match variant.name {
        "Whole" => Ok(SegmentationPolicy::Whole {}),
        "Fixed" => Ok(SegmentationPolicy::Fixed {
            duration: rational(
                decoder,
                variant.fields.get("duration")?,
                &variant.fields.path("duration"),
            )?,
        }),
        "Automatic" => Ok(SegmentationPolicy::Automatic {
            max_segments: decoder.u32(
                variant.fields.get("max_segments")?,
                &variant.fields.path("max_segments"),
            )?,
        }),
        name => Err(unknown_variant(path, "SegmentationPolicy", name)),
    }
}
