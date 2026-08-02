use std::process::Command;

use tempfile::tempdir;
use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_ir::StreamChoice;
use veac_provider::*;

use super::support::*;

#[test]
fn provider_crop_samples_move_the_rendered_subject_window() {
    let temp = tempdir().unwrap();
    let input = subject_fixture(temp.path());
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_reframe",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut clip = media_clip("itm_reframe", "med_reframe", 0, 2_000);
    clip.visual = Some(full_visual());
    canonical.project.sequences[0].tracks.push(track(
        "trk_reframe",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    let assets = BTreeMap::from([("med_reframe".to_owned(), input)]);
    hydrate(&mut canonical, &assets);

    let proposal = reframe_proposal(&canonical);
    assert!(matches!(
        proposal.evidence.as_slice(),
        [ProposalEvidence::CropSamples { .. }]
    ));
    let veac_ir::EditOutcome::Applied {
        project: canonical, ..
    } = veac_ir::apply_edit_batch(&canonical, &proposal.batch)
    else {
        panic!("provider reframe proposal must apply")
    };

    let output = temp.path().join("auto-reframe.mp4");
    let rendered = render(canonical, &assets, &output);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(
        graph.contains("cropzoomv") && graph.contains("eval=frame"),
        "{graph}"
    );
    assert_media_contract(&output, 0, 2.0);
    let early = rgb_at(&output, 0.05, WIDTH / 2, HEIGHT / 2);
    let late = rgb_at(&output, 1.5, WIDTH / 2, HEIGHT / 2);
    assert!(early[0] > 200 && early[2] < 40, "early={early:?}");
    assert!(late[2] > 200 && late[0] < 40, "late={late:?}");
    assert!(
        changed_channels(&rgb_frame(&output, 0.05), &rgb_frame(&output, 1.5))
            > (WIDTH * HEIGHT * 3 / 2) as usize
    );
}

fn reframe_proposal(project: &ProjectEnvelope) -> ProviderEditProposal {
    let material = project
        .project
        .materials
        .iter()
        .find(|value| value.id.as_str() == "med_reframe")
        .unwrap();
    let identity = material.identity.as_ref().unwrap();
    let selection = material
        .probe
        .as_ref()
        .unwrap()
        .selected_video_stream
        .unwrap();
    let input = InputArtifact {
        content: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: identity.digest.clone(),
        },
        media_type: MediaType::Video,
        stream_index: Some(selection.global_index),
        range: Some(TimeRange::new(time(0), time(2_000)).unwrap()),
    };
    let request = ProviderRequestEnvelope::new(
        NegotiatedCapability {
            capability: Capability::AutoReframe,
            contract_version: CAPABILITY_CONTRACT_VERSION,
            provider: fingerprint(),
        },
        ProviderRequest::AutoReframe(AutoReframeRequest {
            video: input,
            target_aspect_ratio: ratio(16, 9),
            subject_tracks: vec![],
            smoothing: 0.5,
            dead_zone: 0.1,
            maximum_speed_per_second: 1.0,
            manual_overrides: vec![],
        }),
    )
    .unwrap();
    let response = ProviderResponseEnvelope::new(
        &request,
        ProviderOutput::AutoReframe(AutoReframeResult {
            crops: vec![crop(0, 0.0), crop(1_000, 0.5)],
            movements: vec![],
            decisions: vec![],
        }),
    )
    .unwrap();
    let context = ApplicationContext::AutoReframe(AutoReframeApplication {
        header: ApplicationHeader {
            project_revision: project.project.revision,
            operation_id: OperationId::new("op_auto_reframe_e2e").unwrap(),
        },
        clip_id: ItemId::new("itm_reframe").unwrap(),
        time: ClipTimeBinding {
            provider_origin: time(0),
            clip_local_origin: time(0),
        },
        keyframe_id_prefix: "kf_auto_reframe_e2e".into(),
    });
    propose_edit(project, &request, &response, &context).unwrap()
}

fn crop(milliseconds: i64, x: f64) -> CropSample {
    CropSample {
        time: time(milliseconds),
        rect: Rect {
            x,
            y: 0.0,
            width: 0.5,
            height: 1.0,
        },
        confidence: 1.0,
    }
}

fn fingerprint() -> ProviderFingerprint {
    ProviderFingerprint {
        provider: "e2e-provider".into(),
        implementation_version: "1".into(),
        model: "reframe".into(),
        model_version: "1".into(),
        configuration: ContentDigest::sha256(b"e2e-reframe"),
    }
}

fn subject_fixture(root: &Path) -> PathBuf {
    let output = root.join("subjects.mp4");
    let result = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", "color=c=red:s=96x54:r=10:d=2"])
        .args(["-f", "lavfi", "-i", "color=c=blue:s=96x54:r=10:d=2"])
        .args([
            "-filter_complex",
            "[0:v][1:v]hstack=inputs=2[v]",
            "-map",
            "[v]",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "fixture failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}
