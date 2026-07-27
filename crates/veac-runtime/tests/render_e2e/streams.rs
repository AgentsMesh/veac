use tempfile::tempdir;
use veac_ir::{ProbedStreamType, StreamChoice};
use veac_runtime::asset::probe_with_intent;

use super::support::*;

#[test]
fn explicit_global_video_selection_excludes_attached_picture_end_to_end() {
    let temp = tempdir().unwrap();
    let input = attached_picture_fixture(temp.path());
    let intent = StreamIntent {
        video: StreamChoice::GlobalIndex { global_index: 0 },
        audio: StreamChoice::Disabled,
    };
    let snapshot = probe_with_intent(&input, intent.clone()).unwrap();
    let attached = snapshot
        .streams
        .iter()
        .find(|stream| stream.disposition.attached_picture)
        .expect("attached picture inventory entry");
    assert_eq!(attached.media_type, ProbedStreamType::Video);
    assert_eq!(attached.global_index, 2);
    assert_eq!(snapshot.selected_video_stream.unwrap().global_index, 0);
    assert_ne!(snapshot.selected_video_stream.unwrap().global_index, 2);

    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_camera",
        MaterialKind::Video,
        intent.video,
        intent.audio,
    ));
    canonical.project.sequences[0].tracks.push(track(
        "trk_camera",
        TrackKind::Video,
        0,
        vec![media_clip("itm_camera", "med_camera", 0, 1_000)],
    ));
    let assets = BTreeMap::from([("med_camera".to_owned(), input)]);
    let output = temp.path().join("selected.mp4");
    let rendered = render(canonical, &assets, &output);

    let selected = rendered.plan.inputs[0].video.as_ref().unwrap().selection;
    assert_eq!((selected.global_index, selected.type_index), (0, 0));
    let graph = rendered.command.filter_graph.unwrap();
    assert!(graph.contains("[0:0]"), "{graph}");
    assert!(!graph.contains("[0:2]"), "{graph}");
    assert_media_contract(&output, 0, 1.0);
    let center = rgb_at(&output, 0.5, WIDTH / 2, HEIGHT / 2);
    assert!(center[2] > 180 && center[0] < 40, "center={center:?}");
}
