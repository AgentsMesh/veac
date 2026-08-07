use std::path::Path;

pub(super) fn replacement(
    name: &str,
    path: &Path,
    probe: veac_ir::MediaProbeSnapshot,
    kind: veac_ir::MaterialKind,
) -> veac_ir::Material {
    veac_ir::Material {
        id: veac_ir::MaterialId::new(format!("med_{name}")).unwrap(),
        kind,
        source: veac_ir::MaterialSource::File {
            uri: path.file_name().unwrap().to_str().unwrap().to_owned(),
        },
        identity: Some(probe.observed_identity.clone()),
        stream_intent: veac_ir::StreamIntent {
            video: veac_ir::StreamChoice::Auto,
            audio: veac_ir::StreamChoice::Disabled,
        },
        probe: Some(probe),
        authorship: None,
    }
}
