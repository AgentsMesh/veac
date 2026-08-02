use veac_artifact::ArtifactKind;

use crate::test_support::*;
use crate::*;

#[test]
fn multi_item_contracts_enforce_stable_ordering() {
    let mut request_values = requests();

    let mut translation = take_translation(&mut request_values);
    let mut unit = translation.units[0].clone();
    unit.id = "unit-2".into();
    unit.range = Some(range(10, 10));
    translation.units.push(unit);
    translation.validate().unwrap();

    let mut dubbing = take_dubbing(&mut request_values);
    let mut turn = dubbing.turns[0].clone();
    turn.id = "turn-2".into();
    turn.source_range = range(10, 10);
    dubbing.turns.push(turn);
    dubbing.validate().unwrap();

    let mut segmentation = take_segmentation(&mut request_values);
    let mut subject = segmentation.subjects[0].clone();
    subject.id = "subject-2".into();
    segmentation.subjects.push(subject);
    segmentation.validate().unwrap();

    let mut filler = take_filler(&mut request_values);
    filler.filler_lexicon = vec!["er".into(), "um".into()];
    filler.validate().unwrap();

    let mut retouch = take_retouch(&mut request_values);
    let mut target = retouch.targets[0].clone();
    target.id = "face-2".into();
    retouch.targets.push(target);
    retouch.controls.push(ParameterCurve {
        parameter: "whitening".into(),
        samples: vec![scalar()],
    });
    retouch.validate().unwrap();

    let mut removal = take_removal(&mut request_values);
    let mut target = removal.targets[0].clone();
    target.id = "logo-2".into();
    removal.targets.push(target);
    removal.validate().unwrap();
}

#[test]
fn multi_item_results_enforce_stable_ordering() {
    let payload = requests().remove(0);
    let request = ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
    let hash = request_hash(&request).unwrap();

    let language = LanguageDetectionResult {
        languages: vec![
            LanguageScore {
                language: "en".into(),
                confidence: 0.9,
            },
            LanguageScore {
                language: "zh".into(),
                confidence: 0.8,
            },
        ],
    };
    language.validate().unwrap();

    let mut output_values = outputs();
    extend_translation(&mut output_values);
    extend_dubbing(&mut output_values);
    extend_segmentation(&mut output_values);

    let mut separation = take_separation(&mut output_values);
    separation.stems.push(SeparatedStem {
        kind: StemKind::Music,
        label: "music".into(),
        audio: artifact(ArtifactKind::AudioStem, "music", &request.provider, hash),
        leakage: 0.1,
    });
    separation.validate().unwrap();
}

fn outputs() -> Vec<ProviderOutput> {
    requests()
        .into_iter()
        .map(|payload| {
            let request =
                ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
            output_for(&request)
        })
        .collect()
}

fn extend_translation(values: &mut [ProviderOutput]) {
    let ProviderOutput::Translation(value) = &mut values[2] else {
        unreachable!()
    };
    let mut unit = value.units[0].clone();
    unit.id = "unit-2".into();
    unit.range = Some(range(10, 10));
    value.units.push(unit);
    value.validate().unwrap();
}

fn extend_dubbing(values: &mut [ProviderOutput]) {
    let ProviderOutput::Dubbing(value) = &mut values[4] else {
        unreachable!()
    };
    let mut turn = value.turns[0].clone();
    turn.id = "turn-2".into();
    turn.source_range = range(10, 10);
    turn.rendered_range = range(10, 10);
    value.turns.push(turn);
    value.validate().unwrap();
}

fn extend_segmentation(values: &mut [ProviderOutput]) {
    let ProviderOutput::Segmentation(value) = &mut values[7] else {
        unreachable!()
    };
    let mut subject = value.subjects[0].clone();
    subject.subject_id = "subject-2".into();
    subject.visible_ranges = vec![range(10, 10)];
    value.subjects.push(subject);
    value.validate().unwrap();
}

fn take_translation(values: &mut Vec<ProviderRequest>) -> TranslationRequest {
    match values.remove(2) {
        ProviderRequest::Translation(value) => value,
        _ => unreachable!(),
    }
}

fn take_dubbing(values: &mut Vec<ProviderRequest>) -> DubbingRequest {
    match values.remove(3) {
        ProviderRequest::Dubbing(value) => value,
        _ => unreachable!(),
    }
}

fn take_segmentation(values: &mut Vec<ProviderRequest>) -> SegmentationRequest {
    match values.remove(5) {
        ProviderRequest::Segmentation(value) => value,
        _ => unreachable!(),
    }
}

fn take_filler(values: &mut Vec<ProviderRequest>) -> FillerDetectionRequest {
    match values.remove(11) {
        ProviderRequest::FillerDetection(value) => value,
        _ => unreachable!(),
    }
}

fn take_retouch(values: &mut Vec<ProviderRequest>) -> RetouchRequest {
    match values.remove(13) {
        ProviderRequest::Retouch(value) => value,
        _ => unreachable!(),
    }
}

fn take_removal(values: &mut Vec<ProviderRequest>) -> RemovalRequest {
    match values.remove(13) {
        ProviderRequest::Removal(value) => value,
        _ => unreachable!(),
    }
}

fn take_separation(values: &mut Vec<ProviderOutput>) -> VocalSeparationResult {
    match values.remove(10) {
        ProviderOutput::VocalSeparation(value) => value,
        _ => unreachable!(),
    }
}
