use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use veac_ir::{Annotation, Material, MulticamGroup, Project, Relation, Sequence};

use crate::{OtioError, OtioTimeline};

pub(crate) const KEY: &str = "veac";
const VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VeacExtension {
    pub version: u32,
    pub projection_sha256: String,
    pub sequence: Sequence,
    pub relations: Vec<Relation>,
    pub materials: Vec<Material>,
    pub multicam_groups: Vec<MulticamGroup>,
    pub annotations: Vec<Annotation>,
}

pub(super) fn attach(
    timeline: &mut OtioTimeline,
    project: &Project,
    sequence: &Sequence,
    relations: Vec<Relation>,
    annotations: Vec<Annotation>,
) -> Result<(), OtioError> {
    let extension = VeacExtension {
        version: VERSION,
        projection_sha256: projection_hash(timeline)?,
        sequence: sequence.clone(),
        relations,
        materials: project.materials.clone(),
        multicam_groups: project.multicam_groups.clone(),
        annotations,
    };
    let value = serde_json::to_value(extension)?;
    timeline.metadata.insert(KEY.to_owned(), value);
    Ok(())
}

pub(crate) fn decode(value: &Value) -> Result<VeacExtension, OtioError> {
    if value.get("version").and_then(Value::as_u64) != Some(u64::from(VERSION)) {
        return Err(OtioError::contract(
            "unsupported VEAC OTIO extension version",
        ));
    }
    let extension: VeacExtension = serde_json::from_value(value.clone())?;
    Ok(extension)
}

pub(crate) fn verify(timeline: &OtioTimeline, extension: &VeacExtension) -> Result<(), OtioError> {
    let actual = projection_hash(timeline)?;
    if actual == extension.projection_sha256 {
        Ok(())
    } else {
        Err(OtioError::contract(
            "VEAC extension is stale; import the edited standard projection with bindings",
        ))
    }
}

fn projection_hash(timeline: &OtioTimeline) -> Result<String, OtioError> {
    let mut projection = timeline.clone();
    projection.metadata.remove(KEY);
    let bytes = serde_json_canonicalizer::to_vec(&projection)?;
    Ok(hex(Sha256::digest(bytes)))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    bytes
        .as_ref()
        .iter()
        .flat_map(|byte| [HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]])
        .map(char::from)
        .collect()
}
