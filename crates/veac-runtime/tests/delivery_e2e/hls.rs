use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_ir::StreamChoice;

use super::hls_probe::{assert_segment_decodes, inspect};
use super::support::*;

#[test]
fn hls_is_a_closed_decodable_atomic_tree_and_tampering_misses_cache() {
    let temp = tempdir().unwrap();
    let tone = tone_fixture(temp.path(), "hls-tone", 440);
    let delivery = prepare_delivery(
        hls_project(),
        &BTreeMap::from([("med_hls_tone".to_owned(), tone)]),
        temp.path(),
    );
    let root = delivery.path("dlv_hls");
    std::fs::create_dir(root).unwrap();
    std::fs::write(root.join("old-only.ts"), b"old package").unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));

    let first = delivery.execute(&store);
    assert!(!first.tasks[0].cache_hit);
    assert_eq!(first.tasks[0].outputs, vec![root.to_path_buf()]);
    assert!(!root.join("old-only.ts").exists());
    let graph = inspect(root);
    assert_eq!(graph.media_playlists.len(), 2);
    assert!(graph.segments.len() >= 4);
    for segment in &graph.segments {
        assert_segment_decodes(segment, true);
    }
    assert!(delivery.execute(&store).tasks[0].cache_hit);

    let tampered = graph.segments[0].clone();
    std::fs::write(&tampered, b"tampered segment").unwrap();
    let repaired = delivery.execute(&store);
    assert!(!repaired.tasks[0].cache_hit);
    let repaired_graph = inspect(root);
    for segment in &repaired_graph.segments {
        assert_segment_decodes(segment, true);
    }
    assert_no_staging_residue(temp.path());
}

pub(super) fn hls_project() -> ProjectEnvelope {
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_hls_tone",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let audio = [0, 1_000]
        .into_iter()
        .enumerate()
        .map(|(index, start)| {
            let mut clip = media_clip(
                &format!("itm_hls_audio_{index}"),
                "med_hls_tone",
                start,
                1_000,
            );
            clip.audio = Some(audio_properties(0.4));
            clip
        })
        .collect();
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_hls_video",
            TrackKind::Video,
            0,
            vec![
                solid_clip("itm_hls_red", color(200, 20, 30), 0, 1_000),
                solid_clip("itm_hls_blue", color(20, 40, 200), 1_000, 1_000),
            ],
        ),
        track("trk_hls_audio", TrackKind::Audio, 1, audio),
    ]);
    canonical.project.render_configs[0].deliverables = vec![hls_deliverable()];
    canonical
}

fn hls_deliverable() -> Deliverable {
    Deliverable {
        id: DeliverableId::new("dlv_hls").unwrap(),
        target: DeliverableTarget::Package {
            name: "stream".into(),
        },
        kind: DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(HlsPackage {
            segment_duration: time(1_000),
            audio: Some(HlsAudio {
                source: AudioMixSource::Master,
                encoding: HlsAudioEncoding::Aac(AacEncoding {
                    bitrate_bps: 64_000,
                    sample_rate_hz: 48_000,
                    channel_layout: AudioChannelLayout::Mono,
                }),
            }),
            renditions: vec![
                rendition("rnd_full", WIDTH, HEIGHT),
                rendition("rnd_small", 64, 36),
            ],
        })),
    }
}

fn rendition(id: &str, width: u32, height: u32) -> HlsRendition {
    HlsRendition {
        id: HlsRenditionId::new(id).unwrap(),
        raster: HlsRenditionRaster { width, height },
        encoding: HlsVideoEncoding::H264(HlsH264Encoding {
            rate_control: HlsCappedBitrate {
                target_bps: 300_000,
                max_bps: 450_000,
                buffer_size_bits: 600_000,
            },
            profile: Some(HlsH264Profile::Main),
            level: None,
            color_space: None,
            b_frames: Some(2),
        }),
    }
}

pub(super) fn assert_no_staging_residue(parent: &Path) {
    let residue = std::fs::read_dir(parent)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".veac-stage-"))
        .collect::<Vec<_>>();
    assert!(residue.is_empty(), "staging residue: {residue:?}");
}
