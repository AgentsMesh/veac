use tempfile::tempdir;
use veac_artifact::{ArtifactStore, ExecutionBindings};
use veac_codegen::emitter;
use veac_ir::{MaterialKind, StreamChoice};
use veac_plan::resolve_one;
use veac_runtime::{asset, executor};

use super::support::*;

#[test]
fn undeclared_hls_children_are_rejected_by_real_ffmpeg() {
    let temp = tempdir().unwrap();
    let reference = color_video_fixture(temp.path(), "reference", "red");
    let manifest = temp.path().join("undeclared.m3u8");
    std::fs::write(
        &manifest,
        "#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1.0,\nmissing-child.ts\n#EXT-X-ENDLIST\n",
    )
    .unwrap();

    let mut probe = asset::probe(&reference).unwrap();
    let identity = asset::sha256_identity(&manifest).unwrap();
    probe.observed_identity = identity.clone();
    let mut source = material(
        "med_hls",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    );
    source.identity = Some(identity);
    source.probe = Some(probe);

    let mut canonical = project(false);
    canonical.project.materials.push(source);
    canonical.project.sequences[0].tracks.push(track(
        "trk_video",
        TrackKind::Video,
        0,
        vec![media_clip("itm_video", "med_hls", 0, 1_000)],
    ));
    let config = canonical.project.render_configs[0].id.clone();
    let plan = resolve_one(&canonical, &config).unwrap();
    let mut bindings = ExecutionBindings::from_originals(
        &plan,
        &BTreeMap::from([(plan.inputs[0].id.clone(), manifest)]),
    )
    .unwrap();
    let output = temp.path().join("must-not-render.mp4");
    bindings
        .bind_output(plan.output.deliverables[0].id.clone(), output.clone())
        .unwrap();
    let bundle = emitter::emit_all(&plan, &bindings).unwrap();

    let error = executor::execute_bundle(&bundle, &ArtifactStore::new(temp.path().join("store")))
        .unwrap_err();
    assert!(error.message.contains("whitelist"), "{}", error.message);
    assert!(error.message.contains("hls"), "{}", error.message);
    assert!(!output.exists());
}
