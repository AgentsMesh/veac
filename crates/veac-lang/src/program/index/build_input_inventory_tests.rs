use crate::program::{prepare_source, SourceIndexBuildInputType};

#[test]
fn executable_inventory_describes_every_primitive_build_input() {
    let source = format!(
        "input parameter a_angle: angle;\n\
         input parameter b_bool: bool;\n\
         input parameter c_color: color;\n\
         input parameter d_int: int;\n\
         input parameter e_length: length;\n\
         input parameter f_scalar: scalar;\n\
         input parameter g_text: text;\n\
         input parameter h_time: time;\n{PROJECT}"
    );
    let prepared = prepare_source(&source).unwrap();
    let inventory = prepared.source_inventory().unwrap();
    let found = inventory
        .build_inputs
        .into_iter()
        .map(|input| input.value_type)
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        vec![
            SourceIndexBuildInputType::Angle,
            SourceIndexBuildInputType::Bool,
            SourceIndexBuildInputType::Color,
            SourceIndexBuildInputType::Integer,
            SourceIndexBuildInputType::Length,
            SourceIndexBuildInputType::Scalar,
            SourceIndexBuildInputType::Text,
            SourceIndexBuildInputType::Time,
        ]
    );
}

#[test]
fn built_program_preserves_the_same_build_input_inventory() {
    let source = format!("input parameter enabled: bool;\n{PROJECT}");
    let prepared = prepare_source(&source).unwrap();
    let expected = prepared.source_inventory().unwrap();
    let mut inputs = crate::program::BuildInputManifestV1::empty();
    inputs.inputs.push(crate::program::BuildInputBinding {
        name: "enabled".into(),
        value: crate::program::BuildInputManifestValue::Bool { value: true },
    });
    let built = prepared.execute_with_inputs(&inputs).unwrap();
    assert_eq!(built.source_inventory().unwrap(), expected);
}

const PROJECT: &str = r#"fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), "interface",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000));
  project(identifier("interface"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}"#;
