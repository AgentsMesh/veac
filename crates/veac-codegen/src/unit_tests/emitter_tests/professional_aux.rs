use veac_codegen::emitter::{emit_all, BackendAction, BackendRequirement};
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved};

#[test]
fn image_sequences_and_pcm_stems_emit_probed_delivery_contracts() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables = vec![
        image("dlv_exr", "frame-%d.exr", ImageFormat::Exr),
        stem("dlv_pcm24", "pcm24.wav", AudioCodec::PcmS24Le),
        stem("dlv_pcm32", "pcm32.wav", AudioCodec::PcmS32Le),
        image("dlv_tiff", "frame-%d.tiff", ImageFormat::Tiff),
    ];
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    let exr = arguments(&bundle, "dlv_exr");
    for (name, value) in [
        ("-c:v", "exr"),
        ("-pix_fmt", "gbrapf32le"),
        ("-compression", "zip16"),
        ("-format", "half"),
    ] {
        assert!(pair(exr, name, value));
    }
    let tiff = arguments(&bundle, "dlv_tiff");
    for (name, value) in [
        ("-c:v", "tiff"),
        ("-pix_fmt", "rgba64le"),
        ("-compression_algo", "deflate"),
    ] {
        assert!(pair(tiff, name, value));
    }
    assert!(pair(arguments(&bundle, "dlv_pcm24"), "-c:a", "pcm_s24le"));
    assert!(pair(arguments(&bundle, "dlv_pcm32"), "-c:a", "pcm_s32le"));
    for name in ["exr", "tiff", "pcm_s24le", "pcm_s32le"] {
        assert!(bundle.requirements().iter().any(|value| matches!(
            value,
            BackendRequirement::Encoder { name: value, .. } if value == name
        )));
    }
    for (id, name) in [
        ("dlv_exr", "image2"),
        ("dlv_tiff", "image2"),
        ("dlv_pcm24", "wav"),
        ("dlv_pcm32", "wav"),
    ] {
        assert!(bundle.requirements().iter().any(|value| {
            value.deliverable_id().as_str() == id
                && value.kind().as_str() == "muxer"
                && value.name() == name
        }));
    }
}

fn image(id: &str, file: &str, format: ImageFormat) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number: 1,
        }),
    }
}

fn stem(id: &str, file: &str, codec: AudioCodec) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file.to_owned(),
        kind: DeliverableKind::AudioStem(AudioStemOutput {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec,
                sample_rate: 96_000,
                channels: 2,
            },
            source: AudioStemSource::Master,
        }),
    }
}

fn arguments<'a>(bundle: &'a veac_codegen::emitter::BackendBundle, id: &str) -> &'a [String] {
    let task = bundle
        .tasks()
        .iter()
        .find(|value| value.deliverable_id.as_str() == id)
        .unwrap();
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    &command.output_args
}

fn pair(values: &[String], name: &str, value: &str) -> bool {
    values.windows(2).any(|pair| pair == [name, value])
}
