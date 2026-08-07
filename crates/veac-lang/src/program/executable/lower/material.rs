use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{
    HashAlgorithm, Material, MaterialKind, MaterialSource, MediaIdentity, StreamChoice,
    StreamIntent,
};

use super::error::ExecutableLowerError;
use super::{id, metadata, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
) -> Result<Material, ExecutableLowerError> {
    if entity.domain_type() != Some(DomainType::Resource) {
        return Err(malformed());
    }
    let (operation, operands) = entity.constructor().ok_or_else(malformed)?;
    let kind = kind(operation)?;
    let path = entity.logical_path().ok_or_else(malformed)?;
    Ok(Material {
        id: id::material(&path),
        kind,
        source: location(graph, operands.get(1).ok_or_else(malformed)?)?,
        identity: Some(identity(graph, operands.get(2).ok_or_else(malformed)?)?),
        stream_intent: streams(graph, operation, operands.get(3))?,
        probe: None,
        authorship: Some(metadata::entity(entity)?),
    })
}

fn kind(operation: Op) -> Result<MaterialKind, ExecutableLowerError> {
    match operation {
        Op::VideoResource => Ok(MaterialKind::Video),
        Op::AudioResource => Ok(MaterialKind::Audio),
        Op::ImageResource => Ok(MaterialKind::Image),
        Op::FontResource => Ok(MaterialKind::Font),
        Op::Lut1dResource => Ok(MaterialKind::Lut1d),
        Op::Lut3dResource => Ok(MaterialKind::Lut3d),
        _ => Err(malformed()),
    }
}

fn location(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<MaterialSource, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ResourceFile, [path]) => Ok(MaterialSource::File {
            uri: value::text(Some(path))?.to_owned(),
        }),
        (Op::ResourceRemoteHttp, [url]) => Ok(MaterialSource::Remote {
            uri: value::text(Some(url))?.to_owned(),
        }),
        _ => Err(malformed()),
    }
}

fn identity(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<MediaIdentity, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::Sha256)?;
    let digest = value::text(operands.first())?;
    let valid = digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(ExecutableLowerError::lower(
            "EXECUTABLE_LOWER_RESOURCE_IDENTITY",
            "an executable resource requires a lowercase hexadecimal SHA-256 digest",
        ));
    }
    Ok(MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: digest.to_owned(),
    })
}

fn streams(
    graph: &FrozenDomainGraph,
    operation: Op,
    authored: Option<&Value>,
) -> Result<StreamIntent, ExecutableLowerError> {
    let disabled = StreamChoice::Disabled;
    let auto = StreamChoice::Auto;
    match operation {
        Op::VideoResource => {
            let operands = value::description_operands(
                graph,
                authored.ok_or_else(malformed)?,
                Op::StreamIntent,
            )?;
            Ok(StreamIntent {
                video: stream(graph, operands.first().ok_or_else(malformed)?)?,
                audio: stream(graph, operands.get(1).ok_or_else(malformed)?)?,
            })
        }
        Op::AudioResource => Ok(StreamIntent {
            video: disabled,
            audio: stream(graph, authored.ok_or_else(malformed)?)?,
        }),
        Op::ImageResource => Ok(StreamIntent {
            video: auto,
            audio: disabled,
        }),
        Op::FontResource | Op::Lut1dResource | Op::Lut3dResource => Ok(StreamIntent {
            video: disabled,
            audio: disabled,
        }),
        _ => Err(malformed()),
    }
}

fn stream(graph: &FrozenDomainGraph, source: &Value) -> Result<StreamChoice, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::StreamAuto, []) => Ok(StreamChoice::Auto),
        (Op::StreamDisabled, []) => Ok(StreamChoice::Disabled),
        (Op::StreamGlobal, [index]) => {
            let global_index =
                u32::try_from(value::integer(Some(index))?).map_err(|_| malformed())?;
            Ok(StreamChoice::GlobalIndex { global_index })
        }
        _ => Err(malformed()),
    }
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable resource has invalid typed ownership or value topology")
}
