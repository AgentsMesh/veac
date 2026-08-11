mod support;

use std::collections::BTreeMap;

use support::{assert_code, manifest, target_ref};
use veac_project::*;

#[test]
fn target_profile_and_dependency_references_must_exist() {
    let mut value = manifest();
    value.targets[0].profiles.push(ProfileId::from("missing"));
    value.targets[1]
        .needs
        .push(target_ref("missing", None, InstanceSelector::Same {}));
    assert_code(&value, IssueCode::UnknownReference);

    let mut profile = manifest();
    profile.targets[1].needs.push(target_ref(
        "plate",
        Some("missing"),
        InstanceSelector::Same {},
    ));
    assert_code(&profile, IssueCode::UnknownReference);
}

#[test]
fn exact_selector_values_are_checked_against_target_matrix() {
    let mut value = manifest();
    value.targets[1].needs.push(target_ref(
        "plate",
        Some("preview"),
        InstanceSelector::Exact {
            locale: Some(LocaleId::from("missing")),
            axes: [(AxisId::from("theme"), AxisValue::from("neon"))]
                .into_iter()
                .collect(),
        },
    ));
    assert_code(&value, IssueCode::UnknownReference);

    let mut nonlocalized = manifest();
    nonlocalized.targets[0].localized = false;
    nonlocalized.targets[1].needs.push(target_ref(
        "plate",
        Some("preview"),
        InstanceSelector::Exact {
            locale: Some(LocaleId::from("zh-cn")),
            axes: BTreeMap::new(),
        },
    ));
    assert_code(&nonlocalized, IssueCode::UnknownReference);
}

#[test]
fn input_bindings_and_artifact_outputs_are_unambiguous() {
    let mut value = manifest();
    let duplicate = value.targets[1].inputs[0].clone();
    value.targets[1].inputs.push(duplicate);
    if let ProjectInputSource::Artifact { output, .. } = &mut value.targets[1].inputs[0].source {
        *output = OutputId::from("missing");
    }
    let error = validate_manifest(&value).unwrap_err();
    let codes = support::codes(&error);
    assert!(codes.contains(&IssueCode::BindingConflict));
    assert!(codes.contains(&IssueCode::UnknownReference));
}

#[test]
fn every_input_source_variant_has_a_strict_contract() {
    let mut value = manifest();
    value.targets[1].inputs = vec![
        ProjectInput {
            id: InputId::from("literal"),
            source: ProjectInputSource::Literal {
                value: ProjectLiteral::Text {
                    value: "hello".to_owned(),
                },
            },
        },
        ProjectInput {
            id: InputId::from("asset-fact"),
            source: ProjectInputSource::AssetFact {
                path: ProjectPath::new("clips/host.mp4"),
                fact: FactId::from("duration"),
            },
        },
        ProjectInput {
            id: InputId::from("analysis"),
            source: ProjectInputSource::AnalysisFact {
                target: target_ref("plate", None, InstanceSelector::AllMatching {}),
                output: OutputId::from("video"),
                fact: FactId::from("loudness"),
            },
        },
    ];
    validate_manifest(&value).unwrap();
    if let ProjectInputSource::AssetFact { path, fact } = &mut value.targets[1].inputs[1].source {
        *path = ProjectPath::new("../host.mp4");
        *fact = FactId::from("Bad_fact");
    }
    assert_code(&value, IssueCode::InvalidPath);
    assert_code(&value, IssueCode::InvalidId);
}

#[test]
fn outputs_and_deliveries_have_typed_unique_ids_and_known_bindings() {
    let mut value = manifest();
    value.targets[0].outputs.extend([
        ProjectOutput::Data {
            id: OutputId::from("report"),
            schema: Some("veac.report/v1".to_owned()),
        },
        ProjectOutput::Directory {
            id: OutputId::from("frames"),
        },
        support::media_output("video"),
    ]);
    value.targets[0]
        .deliveries
        .push(ProjectDelivery::Directory {
            id: DeliveryId::from("plate-file"),
            output: OutputId::from("missing"),
            destination: DeliveryPathTemplate::new("frames/{profile}/{locale}/{axis.theme}"),
        });
    assert_code(&value, IssueCode::DuplicateId);
    assert_code(&value, IssueCode::UnknownReference);
}

#[test]
fn delivery_placeholders_are_closed_and_dimension_aware() {
    for invalid in [
        "dist/{unknown}.mp4",
        "dist/{axis.missing}.mp4",
        "dist/{target.mp4",
        "dist/target}.mp4",
        "dist/{}.mp4",
        "../dist/{target}.mp4",
    ] {
        let mut value = manifest();
        value.targets[0].deliveries[0] = ProjectDelivery::File {
            id: DeliveryId::from("plate-file"),
            output: OutputId::from("video"),
            destination: DeliveryPathTemplate::new(invalid),
        };
        assert_code(&value, IssueCode::InvalidTemplate);
    }

    let mut no_dimensions = manifest();
    no_dimensions.targets[0].profiles.clear();
    no_dimensions.defaults.profile = None;
    no_dimensions.targets[0].localized = false;
    no_dimensions.targets[0].axes.clear();
    assert_code(&no_dimensions, IssueCode::InvalidTemplate);
}
