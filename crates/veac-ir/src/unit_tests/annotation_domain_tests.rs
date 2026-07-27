use crate::{
    test_support::{multicam_project, sample_project},
    *,
};

#[test]
fn material_intrinsic_spans_and_multicam_targets_use_their_own_domains() {
    let material = AnnotationTarget::Material {
        material_id: MaterialId::new("med_video").unwrap(),
    };
    let mut project = sample_project();
    project.project.annotations = vec![
        annotation(
            "ann_material_point",
            material.clone(),
            AnnotationSpan::Point {
                at: RationalTime::new(1, 1_000).unwrap(),
            },
            AnnotationPayload::SceneBoundary {
                confidence: 1.0,
                hard_cut: false,
            },
        ),
        annotation(
            "ann_material_range",
            material,
            AnnotationSpan::Range {
                range: TimeRange::new(
                    RationalTime::new(1, 1_000).unwrap(),
                    RationalTime::new(250, 1_000).unwrap(),
                )
                .unwrap(),
            },
            AnnotationPayload::Scene,
        ),
    ];
    validate(&project).unwrap();

    let mut multicam = multicam_project();
    multicam.project.annotations = vec![annotation(
        "ann_multicam",
        AnnotationTarget::MulticamGroup {
            group_id: MulticamGroupId::new("mcg_interview").unwrap(),
        },
        AnnotationSpan::Untimed,
        AnnotationPayload::Marker {
            label: "Preferred angle".into(),
            color: None,
        },
    )];
    validate(&multicam).unwrap();
}

fn annotation(
    id: &str,
    target: AnnotationTarget,
    span: AnnotationSpan,
    payload: AnnotationPayload,
) -> Annotation {
    Annotation {
        id: AnnotationId::new(id).unwrap(),
        target,
        span,
        payload,
        provenance: None,
    }
}
