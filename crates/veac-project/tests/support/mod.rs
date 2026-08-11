#![allow(dead_code)]

use veac_project::*;

pub mod derivation;

pub fn manifest() -> ProjectManifestV1 {
    ProjectManifestV1 {
        schema: PROJECT_SCHEMA.to_owned(),
        version: PROJECT_MANIFEST_VERSION,
        id: ProjectId::from("receipt-demo"),
        paths: ProjectPaths {
            source_base: ProjectPath::new("sources"),
            material_root: ProjectPath::new("materials"),
            build_root: ProjectPath::new("build"),
            cache_root: ProjectPath::new(".cache/veac"),
            delivery_root: ProjectPath::new("dist"),
        },
        defaults: ProjectDefaults {
            profile: Some(ProfileId::from("preview")),
            locale: Some(LocaleId::from("zh-cn")),
            max_instances_per_target: 32,
            max_total_instances: 64,
        },
        locales: vec![locale("zh-cn", "zh-CN"), locale("en-us", "en-US")],
        profiles: vec![profile("preview"), profile("production")],
        targets: vec![plate_target(), final_target()],
    }
}

pub fn locale(id: &str, tag: &str) -> ProjectLocale {
    ProjectLocale {
        id: LocaleId::from(id),
        language_tag: tag.to_owned(),
    }
}

pub fn profile(id: &str) -> ProjectProfile {
    ProjectProfile {
        id: ProfileId::from(id),
        execution: ExecutionPolicy::Parallel { max_tasks: 2 },
        proxy: ProxyPolicy::PreferExisting {},
        segmentation: SegmentationPolicy::Whole {},
    }
}

pub fn plate_target() -> ProjectTarget {
    ProjectTarget {
        id: TargetId::from("plate"),
        entry: ProjectTargetEntry::Veac {
            source: ProjectPath::new("veac/plate.veac"),
        },
        localized: true,
        inputs: vec![ProjectInput {
            id: InputId::from("logo"),
            source: ProjectInputSource::ProjectMaterial {
                path: ProjectPath::new("logo.png"),
            },
        }],
        profiles: vec![ProfileId::from("production"), ProfileId::from("preview")],
        axes: vec![MatrixAxis {
            id: AxisId::from("theme"),
            values: vec![AxisValue::from("light"), AxisValue::from("dark")],
        }],
        needs: Vec::new(),
        outputs: vec![media_output("video")],
        deliveries: vec![ProjectDelivery::File {
            id: DeliveryId::from("plate-file"),
            output: OutputId::from("video"),
            destination: DeliveryPathTemplate::new("plates/{profile}/{locale}/{axis.theme}.mp4"),
        }],
    }
}

pub fn final_target() -> ProjectTarget {
    ProjectTarget {
        id: TargetId::from("final"),
        entry: ProjectTargetEntry::Veac {
            source: ProjectPath::new("veac/final.veac"),
        },
        localized: true,
        inputs: vec![ProjectInput {
            id: InputId::from("plates"),
            source: ProjectInputSource::Artifact {
                target: target_ref("plate", None, InstanceSelector::AllMatching {}),
                output: OutputId::from("video"),
            },
        }],
        profiles: vec![ProfileId::from("preview"), ProfileId::from("production")],
        axes: Vec::new(),
        needs: Vec::new(),
        outputs: vec![media_output("video")],
        deliveries: vec![ProjectDelivery::File {
            id: DeliveryId::from("final-file"),
            output: OutputId::from("video"),
            destination: DeliveryPathTemplate::new("final/{profile}/{locale}.mp4"),
        }],
    }
}

pub fn target_ref(target: &str, profile: Option<&str>, selector: InstanceSelector) -> TargetRef {
    TargetRef {
        target: TargetId::from(target),
        profile: profile.map(ProfileId::from),
        selector,
    }
}

pub fn media_output(id: &str) -> ProjectOutput {
    ProjectOutput::Media {
        id: OutputId::from(id),
        media_type: MediaType::Video,
    }
}

pub fn codes(error: &ProjectIssues) -> Vec<IssueCode> {
    error.issues().iter().map(|issue| issue.code).collect()
}

pub fn assert_code(manifest: &ProjectManifestV1, code: IssueCode) {
    let error = validate_manifest(manifest).expect_err("manifest must fail validation");
    assert!(
        codes(&error).contains(&code),
        "issues: {:?}",
        error.issues()
    );
}
