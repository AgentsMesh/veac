use veac_codegen::emitter::{CodegenErrorKind, CodegenErrors};
use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn generated_solid_transparent_and_silence_are_typed() {
    for (generator, marker) in [
        (
            Generator::Solid {
                color: Color {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 4,
                },
            },
            "0x01020304",
        ),
        (Generator::Transparent, "color=c=black@0"),
    ] {
        let mut plan = resolved(&fixture());
        clip(&mut plan).source = ResolvedClipSource::Generated { generator };
        assert!(graph(&plan).contains(marker));
    }
    let mut plan = resolved(&fixture());
    clip(&mut plan).source = ResolvedClipSource::Generated {
        generator: Generator::Silence,
    };
    let error = command(&plan).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, "PLAN_STRUCTURE_INVALID");
}

#[test]
fn nested_sequence_uses_its_canvas_and_entry_conforms_output() {
    let mut plan = resolved(&fixture());
    let mut child = plan.sequences[0].clone();
    child.id = SequenceId::new("seq_child").unwrap();
    child.tracks[0].id = TrackId::new("trk_child_video").unwrap();
    child.tracks[0].clips[0].id = ItemId::new("itm_child_video").unwrap();
    child.settings.width = 320;
    child.settings.height = 180;
    child.settings.frame_rate = Rational::new(24, 1).unwrap();
    let main = &mut plan.sequences[0];
    main.settings.width = 1280;
    main.settings.height = 720;
    main.settings.frame_rate = Rational::new(25, 1).unwrap();
    main.tracks[0].clips[0].source = ResolvedClipSource::Sequence {
        sequence_id: child.id.clone(),
    };
    plan.output.width = 640;
    plan.output.height = 360;
    plan.output.frame_rate = Rational::new(30, 1).unwrap();
    plan.sequences.insert(0, child);
    let graph = graph(&plan);
    for marker in [
        "black@0:s=320x180:r=24/1",
        "pad=640:360:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30/1",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}

fn command(
    plan: &veac_plan::ResolvedRenderPlan,
) -> Result<veac_codegen::emitter::BackendCommand, CodegenErrors> {
    emit_video_command(plan, &bindings(plan))
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    command(plan).unwrap().filter_graph.unwrap()
}
