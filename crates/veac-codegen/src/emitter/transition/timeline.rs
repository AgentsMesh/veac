use veac_plan::ResolvedTransition;

use super::super::{time, EmitContext};
use super::kind;

pub(super) fn render(
    context: &mut EmitContext<'_>,
    outgoing: &str,
    incoming: &str,
    transition: &ResolvedTransition,
) -> String {
    let duration = time::seconds_at_least_one_frame(
        transition.record_window.duration,
        context.canvas.frame_rate,
    );
    let mixed = context.graph.filter(
        &[outgoing, incoming],
        format!("{},format=rgba", kind::filter(&transition.kind, &duration)),
        "transitionv",
    );
    let mixed = context.graph.filter(
        &[&mixed],
        format!(
            "setpts=N*{}/({}*TB)",
            context.canvas.frame_rate.denominator, context.canvas.frame_rate.numerator
        ),
        "transitionrebasev",
    );
    let held = context
        .graph
        .filter(&[&mixed], "tpad=stop_mode=clone:stop=-1", "transitionholdv");
    context.graph.filter(
        &[&held],
        format!(
            "setpts=PTS+{}/TB",
            time::seconds(transition.record_window.start)
        ),
        "transitionoffsetv",
    )
}
