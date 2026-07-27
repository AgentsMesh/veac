use super::EmitContext;

pub(super) fn apply(
    context: &mut EmitContext<'_>,
    color: &str,
    alpha: &str,
    format: &str,
    prefix: &str,
) -> String {
    context.graph.filter(
        &[color, alpha],
        format!(
            "mergeplanes=format={format}:map0s=0:map0p=0:map1s=0:map1p=1:map2s=0:map2p=2:map3s=1:map3p=0"
        ),
        prefix,
    )
}
