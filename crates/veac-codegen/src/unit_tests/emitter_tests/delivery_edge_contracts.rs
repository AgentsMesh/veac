use std::path::PathBuf;

use veac_codegen::emitter::{emit_all, BackendAction, BackendProduct};
use veac_plan::canonical::{DeliverableKind, ImageFormat, ImageSequenceOutput};

use super::support::{bindings, fixture, resolved};

#[test]
fn jpeg_image_sequence_emits_numbered_mjpeg_delivery_arguments() {
    let mut plan = resolved(&fixture());
    let mut local = bindings(&plan);
    let deliverable = &mut plan.output.deliverables[0];
    deliverable.file_name = "frame-%d.jpg".into();
    deliverable.kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Jpeg,
        start_number: 17,
    });
    local
        .bind_output(deliverable.id.clone(), PathBuf::from("/tmp/frame-%d.jpg"))
        .unwrap();
    let bundle = emit_all(&plan, &local).unwrap();
    assert_eq!(bundle.tasks().len(), 1);
    assert_eq!(bundle.tasks()[0].product, BackendProduct::ImageSequence);
    let BackendAction::Ffmpeg(command) = &bundle.tasks()[0].action else {
        panic!("image delivery must use FFmpeg")
    };
    for (name, value) in [
        ("-start_number", "17"),
        ("-c:v", "mjpeg"),
        ("-q:v", "2"),
        ("-f", "image2"),
    ] {
        assert!(
            command
                .output_args
                .windows(2)
                .any(|pair| pair == [name, value]),
            "missing {name} {value}: {:?}",
            command.output_args
        );
    }
    assert_eq!(command.output_path, PathBuf::from("/tmp/frame-%d.jpg"));
}
