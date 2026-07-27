use super::*;

#[test]
fn png_and_jpeg_sequences_accept_one_typed_pattern() {
    assert_valid(image("frames-%d.PNG", ImageFormat::Png));
    assert_valid(image("frames-%04d.PNG", ImageFormat::Png));
    assert_valid(image("stills-%d.JpG", ImageFormat::Jpeg));
}

#[test]
fn image_sequences_reject_malformed_patterns_and_format_mismatches() {
    for value in [
        image("frames.png", ImageFormat::Png),
        image("frames-%d-%d.png", ImageFormat::Png),
        image("frames-%00d.png", ImageFormat::Png),
        image("nested/frames-%d.png", ImageFormat::Png),
        image("frames-%d.jpeg", ImageFormat::Jpeg),
        image("frames-%d.jpg", ImageFormat::Png),
        image("frames-%d\0.png", ImageFormat::Png),
    ] {
        assert_invalid(value, "OUTPUT_IMAGE_SEQUENCE");
    }
}

#[test]
fn image_sequence_last_number_must_fit_ffmpeg_integer_domain() {
    let mut value = image("frames-%d.png", ImageFormat::Png);
    let DeliverableKind::ImageSequence(settings) = &mut value.kind else {
        panic!()
    };
    settings.start_number = i32::MAX as u32;
    assert_invalid(value, "OUTPUT_IMAGE_SEQUENCE");
}
