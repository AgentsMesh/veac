use veac_plan::canonical::Mask;

use super::EmitContext;

pub(super) fn apply(context: &mut EmitContext<'_>, input: &str, masks: &[Mask]) -> String {
    let mut label = input.to_owned();
    for mask in masks {
        let alpha = super::mask_expression::alpha(mask);
        label = context.graph.filter(
            &[&label],
            format!("format=rgba,geq=r='r(X\\,Y)':g='g(X\\,Y)':b='b(X\\,Y)':a='{alpha}'"),
            "maskv",
        );
    }
    label
}
