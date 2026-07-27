use veac_ir::{Material, MaterialKind, MediaIdentity};

use crate::{PlanInputId, ResolvedInput, ResolvedInputKind};

pub(super) fn input(
    id: PlanInputId,
    material: &Material,
    canonical_uri: String,
    identity: MediaIdentity,
) -> Option<ResolvedInput> {
    let kind = match material.kind {
        MaterialKind::Font => ResolvedInputKind::Font {
            family: None,
            postscript_name: None,
            face_index: 0,
        },
        MaterialKind::Lut1d | MaterialKind::Lut3d => ResolvedInputKind::Resource {
            material_kind: material.kind,
        },
        _ => return None,
    };
    Some(ResolvedInput {
        id,
        material_id: Some(material.id.clone()),
        kind,
        canonical_uri,
        observed_identity: identity,
        probe: None,
        video: None,
        audio: None,
    })
}
