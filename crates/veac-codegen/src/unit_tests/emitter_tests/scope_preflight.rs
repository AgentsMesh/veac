use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_codegen::emitter::{emit_all, BackendAction, CodegenErrorKind};
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved};

#[test]
fn every_malformed_scope_output_fails_closed_without_panicking() {
    let base = resolved(&fixture());
    let duration = base
        .sequences
        .iter()
        .find(|sequence| sequence.id == base.entry_sequence_id)
        .unwrap()
        .duration;
    let cases = [
        ("zero width", scope(time(0, 600), 0, 360)),
        ("zero height", scope(time(0, 600), 640, 0)),
        ("zero timebase", scope(time(0, 0), 640, 360)),
        ("negative time", scope(time(-1, 600), 640, 360)),
        ("foreign timebase", scope(time(0, 30), 640, 360)),
        ("at sequence end", scope(duration, 640, 360)),
        (
            "after sequence end",
            scope(time(duration.value + 1, 600), 640, 360),
        ),
        ("unsafe time", scope(time(i64::MAX, 600), 640, 360)),
    ];
    for (name, settings) in cases {
        let mut plan = base.clone();
        add_scope(&mut plan, settings);
        assert_scope_invalid(&plan, name);
    }
}

#[test]
fn scope_with_a_missing_entry_sequence_fails_closed() {
    let mut plan = resolved(&fixture());
    plan.entry_sequence_id = SequenceId::new("seq_missing_scope").unwrap();
    add_scope(&mut plan, scope(time(0, 600), 640, 360));
    let error = emit_all(&plan, &bindings(&plan)).expect_err("missing entry");
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|value| value.code == "PLAN_ENTRY_MISMATCH"),
        "{error}"
    );
}

#[test]
fn scope_accepts_the_last_tick_inside_the_entry_sequence() {
    let mut plan = resolved(&fixture());
    let duration = plan.sequences[0].duration;
    add_scope(
        &mut plan,
        scope(time(duration.value - 1, duration.timescale), 640, 360),
    );
    let bundle = emit_all(&plan, &bindings(&plan)).expect("last in-range scope tick");
    let BackendAction::Ffmpeg(command) = &bundle.tasks().last().unwrap().action else {
        panic!("scope must use FFmpeg")
    };
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(
        graph.contains("trim=start_frame=29:end_frame=30"),
        "{graph}"
    );
}

fn add_scope(plan: &mut veac_plan::ResolvedRenderPlan, settings: ScopeOutput) {
    plan.output.deliverables.push(Deliverable {
        id: DeliverableId::new("dlv_scope_preflight").unwrap(),
        target: DeliverableTarget::File {
            name: "scope-preflight.png".to_owned(),
        },
        kind: DeliverableKind::Scope(settings),
    });
    plan.output
        .deliverables
        .sort_by(|left, right| left.id.cmp(&right.id));
}

fn assert_scope_invalid(plan: &veac_plan::ResolvedRenderPlan, name: &str) {
    let result = catch_unwind(AssertUnwindSafe(|| emit_all(plan, &bindings(plan))))
        .unwrap_or_else(|_| panic!("{name} panicked"));
    let error = result.expect_err(name);
    let diagnostic = error
        .diagnostics()
        .iter()
        .find(|value| value.code == "PLAN_SCOPE_OUTPUT_INVALID")
        .unwrap_or_else(|| panic!("{name}: {error}"));
    assert_eq!(diagnostic.kind, CodegenErrorKind::InvalidPlan, "{name}");
}

fn scope(at: RationalTime, width: u32, height: u32) -> ScopeOutput {
    ScopeOutput {
        scope: VideoScope::Waveform,
        at,
        width,
        height,
        format: ImageFormat::Png,
    }
}

fn time(value: i64, timescale: u32) -> RationalTime {
    RationalTime { value, timescale }
}
