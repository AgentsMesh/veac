use tempfile::tempdir;
use veac_ir::{
    CaptionSidecarFormat, CaptionSidecarOutput, ClipSource, Deliverable, DeliverableId,
    DeliverableKind, DeliverableTarget, FontRef, ItemId, Material, MaterialId, MaterialKind,
    MaterialSource, StreamChoice, StreamIntent, TextStyle, TrackId, TrackKind, TrackRouting,
};

use super::super::support::{assert_project_inputs, canonical_project, MEDIA_SOURCE};

#[test]
fn plain_and_styled_sidecars_apply_exact_font_hydration() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("selected.ttf"), b"font").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let base = envelope.project.materials[0].clone();
    envelope.project.materials = vec![
        font(&base, "med_font_selected", "selected.ttf"),
        font(&base, "med_font_unused", "missing.ttf"),
    ];
    let mut selected = caption_track(
        envelope.project.sequences[0].tracks[0].clone(),
        "trk_caption_selected",
        "itm_caption_selected",
        "med_font_selected",
    );
    let mut unused = caption_track(
        selected.clone(),
        "trk_caption_unused",
        "itm_caption_unused",
        "med_font_unused",
    );
    selected.order = 0;
    unused.order = 1;
    envelope.project.sequences[0].tracks = vec![selected, unused];
    envelope.project.render_configs[0].raster = None;
    envelope.project.render_configs[0].deliverables =
        vec![sidecar(CaptionSidecarFormat::WebVtt, "captions.vtt")];

    assert_project_inputs(&project, &envelope, &[], &[], &[]);
    let deliverable = &mut envelope.project.render_configs[0].deliverables[0];
    deliverable.target = DeliverableTarget::File {
        name: "captions.ass".into(),
    };
    let DeliverableKind::CaptionSidecar(settings) = &mut deliverable.kind else {
        unreachable!()
    };
    settings.format = CaptionSidecarFormat::Ass;
    assert_project_inputs(
        &project,
        &envelope,
        &["med_font_selected"],
        &["selected.ttf"],
        &[],
    );
}

fn font(base: &Material, id: &str, uri: &str) -> Material {
    let mut material = base.clone();
    material.id = MaterialId::new(id).unwrap();
    material.kind = MaterialKind::Font;
    material.source = MaterialSource::File { uri: uri.into() };
    material.identity = None;
    material.probe = None;
    material.stream_intent = StreamIntent {
        video: StreamChoice::Disabled,
        audio: StreamChoice::Disabled,
    };
    material
}

fn caption_track(
    mut track: veac_ir::Track,
    track_id: &str,
    item_id: &str,
    font: &str,
) -> veac_ir::Track {
    track.id = TrackId::new(track_id).unwrap();
    track.kind = TrackKind::Caption;
    track.routing = TrackRouting::Default;
    let clip = &mut track.clips[0];
    clip.id = ItemId::new(item_id).unwrap();
    clip.source = ClipSource::Caption {
        text: "caption".into(),
        speaker: None,
        cue: Box::default(),
        style: TextStyle {
            font: FontRef::Material {
                material_id: MaterialId::new(font).unwrap(),
            },
            ..TextStyle::default()
        },
    };
    clip.source_mapping = None;
    clip.audio = None;
    track
}

fn sidecar(format: CaptionSidecarFormat, name: &str) -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_sidecar").unwrap(),
        target: DeliverableTarget::File { name: name.into() },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format,
            track_ids: vec![TrackId::new("trk_caption_selected").unwrap()],
        }),
    }
}
