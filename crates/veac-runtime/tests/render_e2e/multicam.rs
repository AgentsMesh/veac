use std::process::Command;

use tempfile::tempdir;
use veac_plan::ResolvedClipSource;

use super::support::*;

#[test]
fn audio_synced_angles_switch_video_without_emitting_clip_audio() {
    let temp = tempdir().unwrap();
    let blue = color_av_fixture(temp.path(), "blue", "blue");
    let red = color_av_fixture(temp.path(), "red", "red");
    let mut canonical = project(false);
    canonical.project.materials.extend([
        material(
            "med_blue",
            MaterialKind::Video,
            StreamChoice::Auto,
            StreamChoice::Auto,
        ),
        material(
            "med_red",
            MaterialKind::Video,
            StreamChoice::Auto,
            StreamChoice::Auto,
        ),
    ]);
    canonical.project.multicam_groups.push(MulticamGroup {
        id: MulticamGroupId::new("mcg_cameras").unwrap(),
        sync: MulticamSync {
            basis: MulticamSyncBasis::Audio,
            reference_angle_id: MulticamAngleId::new("ang_blue").unwrap(),
        },
        angles: vec![angle("ang_blue", "med_blue"), angle("ang_red", "med_red")],
    });
    let mut clip = solid_clip("itm_multicam", color(0, 0, 0), 0, 2_000);
    clip.source = ClipSource::Multicam {
        group_id: MulticamGroupId::new("mcg_cameras").unwrap(),
        switches: vec![switch("ang_blue", 0), switch("ang_red", 1_000)],
    };
    canonical.project.sequences[0].tracks.push(track(
        "trk_multicam",
        TrackKind::Video,
        0,
        vec![clip],
    ));
    let assets = BTreeMap::from([("med_blue".to_owned(), blue), ("med_red".to_owned(), red)]);
    let output = temp.path().join("multicam.mp4");
    let rendered = render(canonical, &assets, &output);
    let first = rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2);
    let second = rgb_at(&output, 1.5, WIDTH / 2, HEIGHT / 2);
    assert!(first[2] > 180 && first[0] < 60, "first={first:?}");
    assert!(second[0] > 180 && second[2] < 60, "second={second:?}");
    let ResolvedClipSource::Multicam { source } =
        &rendered.plan.sequences[0].tracks[0].clips[0].source
    else {
        unreachable!()
    };
    assert_eq!(source.angles.len(), 2);
    assert!(source
        .angles
        .iter()
        .all(|angle| angle.audio_stream.is_none()));
    assert!(rendered.plan.sequences[0].tracks[0].clips[0]
        .audio
        .is_none());
    assert!(rendered
        .command
        .filter_graph
        .unwrap()
        .contains("concat=n=2:v=1:a=0"));
}

fn color_av_fixture(root: &Path, name: &str, color: &str) -> PathBuf {
    let output = root.join(format!("{name}.mp4"));
    let video = format!("color=c={color}:s=64x64:r=24:d=3");
    let status = Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y"])
        .args(["-f", "lavfi", "-i", &video])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=3"])
        .args(["-map", "0:v:0", "-map", "1:a:0", "-t", "3"])
        .args([
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
        ])
        .args(["-c:a", "aac", "-b:a", "64k"])
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success(), "failed to create {name}");
    output
}

fn angle(id: &str, material: &str) -> MulticamAngle {
    MulticamAngle {
        id: MulticamAngleId::new(id).unwrap(),
        material_id: MaterialId::new(material).unwrap(),
        source_offset: time(0),
    }
}

fn switch(id: &str, start: i64) -> MulticamSwitch {
    MulticamSwitch {
        angle_id: MulticamAngleId::new(id).unwrap(),
        range: TimeRange::new(time(start), time(1_000)).unwrap(),
    }
}
