use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::super::support::TempDir;
use super::binding::replacement;
use super::fixture::{probe, FixtureMedia};

pub(super) const SOURCE: &str = include_str!("source.veac");

pub(super) fn write_request(temp: &TempDir, project: &Path, media: &FixtureMedia) -> PathBuf {
    let envelope = super::fixture::read_project(project);
    let temporal_probe = probe(&media.temporal);
    let portrait_probe = probe(&media.portrait);
    let still_probe = probe(&media.still);
    let clips = &envelope.project.sequences[0].tracks[0].clips;
    assert_eq!(clips.len(), 5);
    let bindings = [
        (
            "fit",
            &media.temporal,
            temporal_probe.clone(),
            veac_ir::MaterialKind::Video,
        ),
        (
            "head",
            &media.temporal,
            temporal_probe.clone(),
            veac_ir::MaterialKind::Video,
        ),
        (
            "center",
            &media.temporal,
            temporal_probe,
            veac_ir::MaterialKind::Video,
        ),
        (
            "crop",
            &media.portrait,
            portrait_probe,
            veac_ir::MaterialKind::Video,
        ),
        (
            "still",
            &media.still,
            still_probe,
            veac_ir::MaterialKind::Image,
        ),
    ]
    .into_iter()
    .zip(clips)
    .map(|((name, path, probe, kind), clip)| {
        let veac_ir::ClipSource::Media { material_id } = &clip.source else {
            panic!("template media slot must reference a material");
        };
        let mut material = replacement(name, path, probe, kind);
        material.id = material_id.clone();
        veac_template::MediaBinding {
            clip_id: clip.id.clone(),
            material,
        }
    })
    .collect();
    let request = veac_template::TemplateFillRequest {
        schema: veac_template::TEMPLATE_FILL_SCHEMA_ID.to_owned(),
        schema_version: veac_template::TEMPLATE_FILL_SCHEMA_VERSION,
        operation_id: veac_ir::OperationId::new("op_template_render_e2e").unwrap(),
        base_revision: envelope.project.revision,
        media_bindings: bindings,
        text_bindings: vec![],
    };
    let output = temp.path().join("template-request.json");
    std::fs::write(&output, serde_json::to_vec_pretty(&request).unwrap()).unwrap();
    output
}

pub(super) fn assert_template_state_cleared(path: &Path) {
    let envelope = super::fixture::read_project(path);
    let sources: BTreeMap<_, _> = envelope
        .project
        .materials
        .iter()
        .map(|material| {
            let veac_ir::MaterialSource::File { uri } = &material.source else {
                panic!("template replacement must remain a local material");
            };
            (uri.as_str(), material.kind)
        })
        .collect();
    for uri in ["temporal.mp4", "portrait.mp4", "still.png"] {
        assert!(sources.contains_key(uri));
    }
    let clips = envelope
        .project
        .sequences
        .iter()
        .flat_map(|sequence| sequence.tracks.iter().flat_map(|track| track.clips.iter()));
    for clip in clips {
        assert!(clip.replaceable.is_none());
        assert!(!clip.template_editable_text);
    }
    assert_eq!(
        envelope
            .project
            .materials
            .iter()
            .find(|material| {
                matches!(
                    &material.source,
                    veac_ir::MaterialSource::File { uri } if uri == "still.png"
                )
            })
            .unwrap()
            .kind,
        veac_ir::MaterialKind::Image
    );
}
