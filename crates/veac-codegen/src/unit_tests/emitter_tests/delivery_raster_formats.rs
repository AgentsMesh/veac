use veac_codegen::emitter::{emit_all, BackendProduct};
use veac_plan::canonical::*;

use super::delivery_extended_support::{ffmpeg, file, pair};
use super::support::{bindings, fixture, resolved, time};

#[test]
fn gif_playback_and_dither_variants_map_to_ffmpeg_semantics() {
    for (index, playback, dither, loop_value, dither_value) in [
        (0, GifPlayback::Once, GifDither::Bayer, "-1", "bayer"),
        (
            1,
            GifPlayback::Forever,
            GifDither::FloydSteinberg,
            "0",
            "floyd_steinberg",
        ),
        (
            2,
            GifPlayback::Times { count: 3 },
            GifDither::Sierra2,
            "2",
            "sierra2",
        ),
        (
            3,
            GifPlayback::Times { count: 2 },
            GifDither::None,
            "1",
            "none",
        ),
    ] {
        let mut plan = resolved(&fixture());
        let id = format!("dlv_gif_{index}");
        plan.output.deliverables = vec![file(
            &id,
            &format!("preview-{index}.gif"),
            DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation { playback, dither })),
        )];
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        assert_eq!(bundle.tasks()[0].product, BackendProduct::AnimatedImage);
        let command = ffmpeg(&bundle, &id);
        assert!(pair(&command.output_args, "-loop", loop_value));
        let graph = command.filter_graph.as_deref().unwrap();
        assert!(graph.contains(&format!("paletteuse=dither={dither_value}")));
    }
}

#[test]
fn still_image_formats_share_exact_containing_frame_selection() {
    for (index, format, extension, encoder, pixel_format) in [
        (0, ImageFormat::Png, "png", "png", "rgba"),
        (1, ImageFormat::Jpeg, "jpg", "mjpeg", "yuvj420p"),
        (2, ImageFormat::Tiff, "tiff", "tiff", "rgba64le"),
        (3, ImageFormat::Exr, "exr", "exr", "gbrapf32le"),
    ] {
        let mut plan = resolved(&fixture());
        let id = format!("dlv_still_{index}");
        plan.output.deliverables = vec![file(
            &id,
            &format!("cover-{index}.{extension}"),
            DeliverableKind::StillImage(StillImage {
                frame: FrameSelection::Containing { at: time(300) },
                encoding: format,
            }),
        )];
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        assert_eq!(bundle.tasks()[0].product, BackendProduct::StillImage);
        let command = ffmpeg(&bundle, &id);
        assert!(pair(&command.output_args, "-frames:v", "1"));
        assert!(pair(&command.output_args, "-c:v", encoder));
        assert!(pair(&command.output_args, "-pix_fmt", pixel_format));
        assert!(command
            .filter_graph
            .as_deref()
            .unwrap()
            .contains("trim=start_frame=15:end_frame=16"));
    }
}
