use std::process::Command;

use tempfile::tempdir;

use super::support::*;

const PERIOD: usize = 12;
const SAMPLES: [f64; 10] = [0.1, 0.3, 0.5, 0.7, 0.9, 1.1, 1.3, 1.5, 1.7, 1.9];

#[test]
fn stabilization_reduces_same_source_motion_without_freezing_content() {
    let temp = tempdir().unwrap();
    let source = shaky_grid_fixture(temp.path());
    let output = temp.path().join("stabilized.mp4");
    let rendered = render(
        stabilization_project(),
        &BTreeMap::from([("med_shaky".to_owned(), source)]),
        &output,
    );
    assert_media_contract(&output, 0, 4.0);
    assert_eq!(rendered.command.preparations.len(), 1);
    let detect = rendered.command.preparations[0]
        .command
        .filter_graph
        .as_deref()
        .unwrap();
    assert!(detect.contains("vidstabdetect=result="), "{detect}");
    assert!(detect.contains("format=yuv444p16le"), "{detect}");
    let transform = rendered.command.filter_graph.as_deref().unwrap();
    assert!(transform.contains("vidstabtransform=input="), "{transform}");
    assert!(transform.contains("format=yuv444p16le"), "{transform}");
    assert!(transform.contains("maxshift=-1"), "{transform}");
    assert!(transform.contains("interpol=2"), "{transform}");

    let before = sampled_frames(&output, 0.0);
    let after = sampled_frames(&output, 2.0);
    let before_phases = phases(&before);
    let after_phases = phases(&after);
    let before_motion = grid_motion(&before_phases, PERIOD);
    let after_motion = grid_motion(&after_phases, PERIOD);
    assert!(before_motion >= 5.0, "source motion={before_motion}");
    assert!(after_motion <= 4.0, "stabilized motion={after_motion}");
    assert!(
        after_motion <= before_motion * 0.45,
        "before={before_motion} after={after_motion}"
    );
    let content = aligned_content_count(
        &after,
        &after_phases,
        WIDTH as usize,
        HEIGHT as usize,
        PERIOD,
    );
    assert!(content >= 4, "stabilized content signatures={content}");
}

fn stabilization_project() -> ProjectEnvelope {
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_shaky",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let mut before = media_clip("itm_shaky_before", "med_shaky", 0, 2_000);
    before.visual = Some(full_visual());
    let mut after = media_clip("itm_shaky_after", "med_shaky", 2_000, 2_000);
    after.visual = Some(full_visual());
    after.effects.push(video_effect(
        "fx_stabilize_e2e",
        Effect::VideoStabilize { enabled: true },
    ));
    canonical.project.sequences[0].tracks.push(track(
        "trk_stabilize",
        TrackKind::Video,
        0,
        vec![before, after],
    ));
    canonical
}

fn sampled_frames(media: &Path, offset: f64) -> Vec<Vec<u8>> {
    SAMPLES
        .iter()
        .map(|second| rgb_frame(media, offset + second))
        .collect()
}

fn phases(frames: &[Vec<u8>]) -> Vec<GridPhase> {
    frames
        .iter()
        .map(|frame| grid_phase(frame, WIDTH as usize, HEIGHT as usize, PERIOD))
        .collect()
}

fn shaky_grid_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("shaky-grid.mp4");
    let source = "testsrc2=s=112x70:r=10:d=2,drawgrid=w=12:h=12:color=white@0.9:t=2,crop=96:54:x=8+5*sin(n*1.7):y=8+5*cos(n*1.3)";
    run(Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            source,
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&output));
    output
}
