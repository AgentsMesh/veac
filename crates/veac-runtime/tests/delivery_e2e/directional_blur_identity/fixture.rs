use std::collections::BTreeSet;
use std::process::Command;

use veac_artifact::ArtifactStore;

use super::super::support::*;

pub(super) const FRAMES: usize = FPS as usize;

pub(super) fn render_frames(
    root: &Path,
    source: &Path,
    name: &str,
    effect: Option<EffectInstance>,
) -> Vec<Vec<u8>> {
    let directory = root.join(name);
    std::fs::create_dir(&directory).unwrap();
    let assets = BTreeMap::from([("med_directional_identity".to_owned(), source.to_path_buf())]);
    let delivery = prepare_delivery(identity_project(effect), &assets, &directory);
    let result = delivery.execute(&ArtifactStore::new(directory.join("store")));
    let mut paths = result.tasks[0].outputs.clone();
    paths.sort();
    assert_eq!(paths.len(), FRAMES);
    paths.iter().map(|path| rgba16(path)).collect()
}

pub(super) fn effect(
    id: &str,
    radius: Animatable<f64>,
    window: Option<(i64, i64)>,
) -> EffectInstance {
    let mut value = video_effect(
        &format!("fx_directional_{id}"),
        Effect::VideoDirectionalBlur {
            angle_degrees: constant(0.0),
            radius,
        },
    );
    value.enable_range =
        window.map(|(start, duration)| TimeRange::new(time(start), time(duration)).unwrap());
    value
}

pub(super) fn zero_crossing() -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: [(0, 0.0), (300, 12.0), (700, 0.0), (999, 0.0)]
            .into_iter()
            .enumerate()
            .map(|(index, (at, value))| Keyframe {
                id: KeyframeId::new(format!("kf_identity_{index}")).unwrap(),
                time: time(at),
                value,
                interpolation: Interpolation::Hold,
            })
            .collect(),
    }
}

pub(super) fn constant(value: f64) -> Animatable<f64> {
    Animatable::constant(value)
}

pub(super) fn alpha_fixture(directory: &Path) -> PathBuf {
    let path = directory.join("directional-alpha.tiff");
    let filter = format!(
        "nullsrc=s={WIDTH}x{HEIGHT},format=gbrap16le,geq=\
         r='1000+mod(X*977+Y*131,50000)':g='2000+mod(X*313+Y*1291,48000)':\
         b='3000+mod(X*1877+Y*173,45000)':a='1024+mod(X*521+Y*733,63000)'"
    );
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
        ])
        .arg(filter)
        .args(["-frames:v", "1", "-pix_fmt", "rgba64le", "-c:v", "tiff"])
        .arg(&path));
    path
}

pub(super) fn assert_high_precision_alpha(frame: &[u8]) {
    let plane = (WIDTH * HEIGHT * 2) as usize;
    let alpha: BTreeSet<_> = frame[plane * 3..]
        .chunks_exact(2)
        .map(|value| u16::from_le_bytes([value[0], value[1]]))
        .collect();
    assert!(alpha.len() > 256);
    assert!(alpha.iter().any(|value| value % 257 != 0));
}

fn identity_project(effect: Option<EffectInstance>) -> ProjectEnvelope {
    let mut value = project(false);
    value.project.materials.push(material(
        "med_directional_identity",
        MaterialKind::Image,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    value.project.sequences[0].tracks.push(track(
        "trk_directional_identity",
        TrackKind::Visual,
        0,
        vec![clip(effect)],
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

fn clip(effect: Option<EffectInstance>) -> Clip {
    let mut value = media_clip(
        "itm_directional_identity",
        "med_directional_identity",
        0,
        1_000,
    );
    value.visual = Some(full_visual());
    value.effects.extend(effect);
    value
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
