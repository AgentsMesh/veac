use super::*;

#[test]
fn duplicate_static_delivery_names_are_rejected() {
    let mut project = sample_project();
    let mut duplicate = project.project.render_configs[0].deliverables[0].clone();
    duplicate.id = DeliverableId::new("dlv_secondary").unwrap();
    project.project.render_configs[0]
        .deliverables
        .push(duplicate);
    assert_code(&validation_codes(&project), "OUTPUT_FILE_COLLISION");
}

#[test]
fn static_output_may_not_be_consumed_by_an_image_pattern() {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables = vec![
        image("dlv_frames", "frame-%d.png", ImageFormat::Png),
        still("dlv_still", "frame-1.png"),
    ];
    assert_code(&validation_codes(&project), "OUTPUT_FILE_COLLISION");
}

#[test]
fn padded_pattern_only_consumes_indices_meeting_its_minimum_width() {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables = vec![
        image("dlv_frames", "frame-%04d.png", ImageFormat::Png),
        still("dlv_still", "frame-1.png"),
    ];
    validate(&project).unwrap();

    project.project.render_configs[0].deliverables[1].file_name = "frame-0001.png".into();
    assert_code(&validation_codes(&project), "OUTPUT_FILE_COLLISION");
}

#[test]
fn intersecting_image_patterns_are_rejected_but_disjoint_patterns_are_valid() {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables = vec![
        image("dlv_frames", "frame-%d.png", ImageFormat::Png),
        image("dlv_nested", "frame-1%d.png", ImageFormat::Png),
    ];
    assert_code(&validation_codes(&project), "OUTPUT_FILE_COLLISION");

    project.project.render_configs[0].deliverables = vec![
        image("dlv_jpeg", "jpeg-%d.jpg", ImageFormat::Jpeg),
        image("dlv_png", "png-%d.png", ImageFormat::Png),
    ];
    validate(&project).unwrap();
}

#[test]
fn image_pattern_requires_exactly_one_placeholder() {
    let mut project = sample_project();
    project.project.render_configs[0].deliverables =
        vec![image("dlv_frames", "frame-%d-%d.png", ImageFormat::Png)];
    let codes = validation_codes(&project);
    assert_code(&codes, "OUTPUT_IMAGE_SEQUENCE");
}

fn image(id: &str, file_name: &str, format: ImageFormat) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file_name.to_owned(),
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number: 1,
        }),
    }
}

fn still(id: &str, file_name: &str) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        file_name: file_name.to_owned(),
        kind: DeliverableKind::Scope(ScopeOutput {
            scope: VideoScope::Waveform,
            at: RationalTime::new(0, 600).unwrap(),
            width: 64,
            height: 64,
            format: ImageFormat::Png,
        }),
    }
}
