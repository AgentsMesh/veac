use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::delivery_probe::{rgb, stream};
use super::support::*;

#[test]
fn all_still_formats_decode_the_exact_containing_timeline_frame() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_still",
        TrackKind::Video,
        0,
        vec![
            solid_clip("itm_red", color(220, 20, 30), 0, 500),
            solid_clip("itm_blue", color(20, 40, 220), 500, 500),
        ],
    ));
    let mut deliverables = [
        ("png", ImageFormat::Png),
        ("jpg", ImageFormat::Jpeg),
        ("tiff", ImageFormat::Tiff),
        ("exr", ImageFormat::Exr),
    ]
    .into_iter()
    .map(|(extension, encoding)| still(extension, encoding))
    .collect::<Vec<_>>();
    deliverables.sort_by(|left, right| left.id.cmp(&right.id));
    canonical.project.render_configs[0].deliverables = deliverables;
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let result = delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    assert_eq!(result.tasks.len(), 4);

    for (extension, codec) in [
        ("png", "png"),
        ("jpg", "mjpeg"),
        ("tiff", "tiff"),
        ("exr", "exr"),
    ] {
        let path = delivery.path(&format!("dlv_{extension}"));
        let video = stream(path, "v:0", "stream=codec_name,width,height");
        assert_eq!(video["codec_name"], codec);
        assert_eq!(video["width"], WIDTH);
        assert_eq!(video["height"], HEIGHT);
        let pixels = rgb(path);
        assert_eq!(pixels.len(), (WIDTH * HEIGHT * 3) as usize);
        let center = (usize::try_from((HEIGHT / 2) * WIDTH + WIDTH / 2).unwrap()) * 3;
        let sample = &pixels[center..center + 3];
        assert!(sample[2] > sample[0] + 80, "{extension} pixel {sample:?}");
    }
}

fn still(extension: &str, encoding: ImageFormat) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(format!("dlv_{extension}")).unwrap(),
        target: DeliverableTarget::File {
            name: format!("cover.{extension}"),
        },
        kind: DeliverableKind::StillImage(StillImage {
            frame: FrameSelection::Containing { at: time(650) },
            encoding,
        }),
    }
}
