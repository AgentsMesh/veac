use tempfile::tempdir;
use veac_artifact::ArtifactStore;
use veac_ir::StreamChoice;

use super::delivery_probe::{gif_loop_count, stream};
use super::support::*;

#[test]
fn mp3_is_a_decodable_exact_rate_master_and_resumes() {
    let temp = tempdir().unwrap();
    let tone = tone_fixture(temp.path(), "podcast-source", 440);
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_tone",
        MaterialKind::Audio,
        StreamChoice::Disabled,
        StreamChoice::Auto,
    ));
    let mut clip = media_clip("itm_tone", "med_tone", 0, 1_000);
    clip.audio = Some(audio_properties(0.5));
    canonical.project.sequences[0]
        .tracks
        .push(track("trk_tone", TrackKind::Audio, 0, vec![clip]));
    canonical.project.render_configs[0].raster = None;
    canonical.project.render_configs[0].deliverables = vec![file(
        "dlv_mp3",
        "podcast.mp3",
        DeliverableKind::AudioFile(AudioFile {
            source: AudioMixSource::Master,
            encoding: AudioFileEncoding::Mp3(Mp3Encoding {
                bitrate_bps: 192_000,
                sample_rate_hz: 48_000,
                channel_layout: AudioChannelLayout::Stereo,
            }),
        }),
    )];
    let assets = BTreeMap::from([("med_tone".to_owned(), tone)]);
    let delivery = prepare_delivery(canonical, &assets, temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));

    let first = delivery.execute(&store);
    assert!(!first.tasks[0].cache_hit);
    let output = delivery.path("dlv_mp3");
    let audio = stream(
        output,
        "a:0",
        "stream=codec_name,sample_rate,channels,bit_rate",
    );
    assert_eq!(audio["codec_name"], "mp3");
    assert_eq!(audio["sample_rate"], "48000");
    assert_eq!(audio["channels"], 2);
    assert_eq!(audio["bit_rate"], "192000");
    assert!(rms_db(&audio_samples(output, 0.1, 0.5)) > -35.0);
    assert!(delivery.execute(&store).tasks[0].cache_hit);
}

#[test]
fn gif_contains_motion_and_the_declared_finite_loop_count() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_motion",
        TrackKind::Video,
        0,
        vec![
            solid_clip("itm_red", color(220, 20, 30), 0, 500),
            solid_clip("itm_blue", color(20, 40, 220), 500, 500),
        ],
    ));
    canonical.project.render_configs[0].deliverables = vec![file(
        "dlv_gif",
        "motion.gif",
        DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
            playback: GifPlayback::Times { count: 3 },
            dither: GifDither::Sierra2,
        })),
    )];
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));

    let output = delivery.path("dlv_gif");
    let video = stream(output, "v:0", "stream=codec_name,width,height,nb_frames");
    assert_eq!(video["codec_name"], "gif");
    assert_eq!(video["width"], WIDTH);
    assert_eq!(video["height"], HEIGHT);
    assert!(video_frame_count(output) >= 8);
    assert_eq!(gif_loop_count(output), 2);
    let early = rgb_at(output, 0.1, WIDTH / 2, HEIGHT / 2);
    let late = rgb_at(output, 0.7, WIDTH / 2, HEIGHT / 2);
    assert!(early[0] > early[2] + 80, "early pixel {early:?}");
    assert!(late[2] > late[0] + 80, "late pixel {late:?}");
}

fn file(id: &str, name: &str, kind: DeliverableKind) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File { name: name.into() },
        kind,
    }
}
