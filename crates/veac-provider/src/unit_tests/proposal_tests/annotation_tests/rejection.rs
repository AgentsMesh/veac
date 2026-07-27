use super::*;

#[test]
fn empty_analysis_results_are_never_silently_applied() {
    for capability in [
        Capability::LanguageDetection,
        Capability::SceneDetection,
        Capability::BeatDetection,
        Capability::SilenceDetection,
        Capability::FillerDetection,
        Capability::HighlightDetection,
    ] {
        let (request, mut response) = exchange(capability);
        clear(&mut response.output);
        assert!(propose_edit(
            &project(),
            &request,
            &response,
            &analysis_context(capability)
        )
        .is_err());
    }
}

#[test]
fn timed_and_untimed_outputs_require_their_exact_time_domain() {
    let (request, response) = exchange(Capability::BeatDetection);
    let mut missing = analysis_context(Capability::BeatDetection);
    application_mut(&mut missing).time = None;
    assert_invalid(propose_edit(&project(), &request, &response, &missing));

    let (request, response) = exchange(Capability::LanguageDetection);
    let mut unexpected = analysis_context(Capability::LanguageDetection);
    application_mut(&mut unexpected).time = Some(AnnotationTimeBinding {
        provider_origin: time(0),
        target_origin: time(0),
    });
    assert_invalid(propose_edit(&project(), &request, &response, &unexpected));

    let (request, response) = exchange(Capability::BeatDetection);
    for target in [
        AnnotationTarget::Project,
        AnnotationTarget::MulticamGroup {
            group_id: MulticamGroupId::new("mcg_missing").unwrap(),
        },
    ] {
        let mut context = analysis_context(Capability::BeatDetection);
        application_mut(&mut context).target = target;
        assert_invalid(propose_edit(&project(), &request, &response, &context));
    }
}

#[test]
fn time_mapping_rejects_negative_and_clip_out_of_bounds_annotations() {
    let (request, response) = exchange(Capability::BeatDetection);
    let mut negative = analysis_context(Capability::BeatDetection);
    let binding = application_mut(&mut negative).time.as_mut().unwrap();
    binding.provider_origin = time(1);
    assert_invalid(propose_edit(&project(), &request, &response, &negative));

    let mut outside = analysis_context(Capability::BeatDetection);
    let value = application_mut(&mut outside);
    value.target = AnnotationTarget::Clip {
        clip_id: ItemId::new("itm_text").unwrap(),
    };
    value.time.as_mut().unwrap().target_origin = time(11);
    assert_invalid(propose_edit(&project(), &request, &response, &outside));
}

#[test]
fn prefixes_collisions_revision_and_capability_mismatches_fail_closed() {
    let (request, response) = exchange(Capability::BeatDetection);
    let mut bad_prefix = analysis_context(Capability::BeatDetection);
    application_mut(&mut bad_prefix).annotation_id_prefix = "bad".into();
    assert_invalid(propose_edit(&project(), &request, &response, &bad_prefix));

    let scene = analysis_case(Capability::SceneDetection);
    let mut collision = scene.project.clone();
    collision
        .project
        .annotations
        .push(annotation(&scene.proposal, 0).clone());
    assert_invalid(propose_edit(
        &collision,
        &scene.request,
        &scene.response,
        &scene.context,
    ));

    let mut stale = analysis_context(Capability::BeatDetection);
    application_mut(&mut stale).header.project_revision += 1;
    assert_invalid(propose_edit(&project(), &request, &response, &stale));

    let mismatch = analysis_context(Capability::SceneDetection);
    assert_invalid(propose_edit(&project(), &request, &response, &mismatch));
}

fn clear(output: &mut ProviderOutput) {
    match output {
        ProviderOutput::LanguageDetection(value) => value.languages.clear(),
        ProviderOutput::SceneDetection(value) => {
            value.boundaries.clear();
            value.scenes.clear();
        }
        ProviderOutput::BeatDetection(value) => value.beats.clear(),
        ProviderOutput::SilenceDetection(value) => value.intervals.clear(),
        ProviderOutput::FillerDetection(value) => {
            value.occurrences.clear();
            value.decisions.clear();
        }
        ProviderOutput::HighlightDetection(value) => {
            value.highlights.clear();
            value.decisions.clear();
        }
        _ => panic!("analysis output fixture"),
    }
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
