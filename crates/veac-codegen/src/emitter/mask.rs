use veac_plan::canonical::Mask;

use super::{process_owner::ProcessOwner, EmitContext};

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    owner: ProcessOwner<'_>,
    input: &str,
    masks: &[Mask],
) -> String {
    let mut label = input.to_owned();
    for mask in masks {
        let alpha = super::mask_expression::alpha(context.plan, owner, mask);
        label = context.graph.filter(
            &[&label],
            format!("format=rgba,geq=r='r(X\\,Y)':g='g(X\\,Y)':b='b(X\\,Y)':a='{alpha}'"),
            "maskv",
        );
    }
    label
}
