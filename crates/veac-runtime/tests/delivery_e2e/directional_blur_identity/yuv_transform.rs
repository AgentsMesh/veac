use std::process::Command;

use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::super::support::*;
use super::fixture::{constant, effect, zero_crossing, FRAMES};

#[test]
fn inactive_blur_is_exact_after_yuv_decode_scale_and_rotation() {
    let temp = tempdir().unwrap();
    let source = yuv_fixture(temp.path());
    assert_order_sensitive(&source);
    let baseline = render_frames(temp.path(), &source, "yuv-baseline", None);
    let zero = render_frames(
        temp.path(),
        &source,
        "yuv-zero",
        Some(effect("yuv-zero", constant(0.0), None)),
    );
    let window = render_frames(
        temp.path(),
        &source,
        "yuv-window",
        Some(effect("yuv-window", constant(12.0), Some((300, 400)))),
    );
    let dynamic = render_frames(
        temp.path(),
        &source,
        "yuv-dynamic",
        Some(effect("yuv-dynamic", zero_crossing(), None)),
    );

    assert!(
        baseline == zero,
        "static zero changed transformed YUV pixels"
    );
    for index in [0, 1, 2, 7, 8, 9] {
        assert!(baseline[index] == window[index], "window frame {index}");
        assert!(baseline[index] == dynamic[index], "dynamic frame {index}");
    }
    for index in 3..7 {
        assert!(baseline[index] != window[index], "window frame {index}");
        assert!(baseline[index] != dynamic[index], "dynamic frame {index}");
    }
}

fn render_frames(
    root: &Path,
    source: &Path,
    name: &str,
    effect: Option<EffectInstance>,
) -> Vec<Vec<u8>> {
    let directory = root.join(name);
    std::fs::create_dir(&directory).unwrap();
    let assets = BTreeMap::from([("med_directional_identity".to_owned(), source.to_path_buf())]);
    let delivery = prepare_delivery(project_with_transform(effect), &assets, &directory);
    let result = delivery.execute(&ArtifactStore::new(directory.join("store")));
    let mut paths = result.tasks[0].outputs.clone();
    paths.sort();
    assert_eq!(paths.len(), FRAMES);
    paths.iter().map(|path| rgba16(path)).collect()
}

fn project_with_transform(effect: Option<EffectInstance>) -> ProjectEnvelope {
    let mut value = project(false);
    value.project.materials.push(material(
        "med_directional_identity",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut clip = media_clip(
        "itm_directional_yuv_transform",
        "med_directional_identity",
        0,
        1_000,
    );
    let mut visual = full_visual();
    visual.transform.scale = Animatable::constant(Vec2 { x: 0.83, y: 1.17 });
    visual.transform.rotation_degrees = Animatable::constant(17.0);
    clip.visual = Some(visual);
    clip.effects.extend(effect);
    value.project.sequences[0].tracks.push(track(
        "trk_directional_yuv_transform",
        TrackKind::Visual,
        0,
        vec![clip],
    ));
    value.project.render_configs[0].deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_main").unwrap(),
        target: DeliverableTarget::ImageSequence {
            pattern: "identity-%02d.tiff".to_owned(),
        },
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format: ImageFormat::Tiff,
            start_number: 1,
        }),
    }];
    value
}

fn yuv_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("directional-yuv.mkv");
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!(
                "testsrc2=s={WIDTH}x{HEIGHT}:r={FPS}:d=1,\
                 drawbox=x=9+3*t:y=7:w=19:h=13:color=0xF02080:t=fill"
            ),
            "-c:v",
            "ffv1",
            "-level",
            "3",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output));
    output
}

fn assert_order_sensitive(source: &Path) {
    let scale = "scale=80:63";
    let rotate = "rotate=17*PI/180:ow=100:oh=100:c=black@0";
    let yuv_first = filtered_frame(source, &format!("{scale},format=gbrap16le,{rotate}"));
    let rgb_first = filtered_frame(source, &format!("format=gbrap16le,{scale},{rotate}"));
    assert_ne!(yuv_first, rgb_first, "fixture hides format-order changes");
}

fn filtered_frame(source: &Path, filter: &str) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(source)
        .args(["-vf", filter, "-frames:v", "1", "-pix_fmt", "gbrap16le"])
        .args(["-f", "rawvideo", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn rgba16(path: &Path) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-pix_fmt",
            "gbrap16le",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), (WIDTH * HEIGHT * 8) as usize);
    output.stdout
}

fn run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
