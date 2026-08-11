use super::*;

#[test]
fn exact_project_rationals_render_without_floating_point() {
    assert_eq!(decimal(ProjectRational::new(3, 2)).unwrap(), "1.5");
    assert_eq!(decimal(ProjectRational::new(-1, 8)).unwrap(), "-0.125");
    assert!(decimal(ProjectRational::new(1, 3)).is_err());
}

#[test]
fn every_project_literal_maps_to_a_closed_build_input_value() {
    let prepared = declaration("text");
    let declaration = &prepared.build_input_declarations()["value"];
    for item in [
        ProjectLiteral::Bool { value: true },
        ProjectLiteral::Integer { value: 7 },
        ProjectLiteral::Scalar {
            value: ProjectRational::new(3, 2),
        },
        ProjectLiteral::Text {
            value: "title".into(),
        },
        ProjectLiteral::Identifier {
            value: "dark".into(),
        },
        ProjectLiteral::Duration {
            value: ProjectRational::new(1, 4),
        },
    ] {
        assert!(literal(&item, declaration).is_ok());
    }
}

#[test]
fn identifier_binding_supports_text_and_payloadless_enums_only() {
    let text = declaration("text");
    assert!(matches!(
        identifier("dark", &text.build_input_declarations()["value"]).unwrap(),
        BuildInputManifestValue::Text { .. }
    ));
    let nominal = declaration("Theme");
    assert!(matches!(
        identifier("Dark", &nominal.build_input_declarations()["value"]).unwrap(),
        BuildInputManifestValue::Enum { .. }
    ));
    let boolean = declaration("bool");
    assert!(identifier("dark", &boolean.build_input_declarations()["value"]).is_err());
}

fn declaration(value_type: &str) -> veac_lang::program::ExecutableBuild {
    let nominal = if value_type == "Theme" {
        "enum Theme { Dark, }\n"
    } else {
        ""
    };
    veac_lang::program::prepare_source(&format!(
        "{nominal}input parameter value: {value_type};\nfn main(context: Context) -> Project {{ project(identifier(\"test\"), project_settings(1000)) }}"
    ))
    .unwrap()
}
