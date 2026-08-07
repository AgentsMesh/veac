use std::path::Path;

use veac_ir::{ClipSource, Generator};
use veac_lang::program::{build_path_with_inputs, parse_build_input_manifest};

#[test]
fn checked_in_example_makes_block_control_flow_visible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let inputs =
        std::fs::read_to_string(root.join("examples/programming-language/build-inputs.json"))
            .unwrap();
    let inputs = parse_build_input_manifest(&inputs).unwrap();
    let built = build_path_with_inputs(
        &root.join("examples/programming-language/main.veac"),
        &inputs,
    )
    .unwrap();
    let envelope = built.envelope();
    assert_eq!(envelope.project.sequences.len(), 1);
    let main = &envelope.project.sequences[0];
    assert_eq!(main.id, envelope.project.entry_sequence_id);
    assert_eq!(main.tracks.len(), 2);
    assert!(main.tracks.iter().all(|track| track.clips.len() == 2));
    let colors = &main.tracks[0].clips;
    assert!(matches!(
        colors[0].source,
        ClipSource::Generated {
            generator: Generator::Solid { .. }
        }
    ));
    let titles = main.tracks[1]
        .clips
        .iter()
        .map(|clip| match &clip.source {
            ClipSource::Text { text, .. } => text.as_str(),
            _ => panic!("expected text clip"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        titles,
        [
            "组件化：结构与方法：静态类型组件",
            "枚举与控制流：穷尽匹配生成"
        ]
    );
    assert_ne!(colors[0].id, colors[1].id);
    assert!(veac_ir::validate(envelope).is_ok());
}
