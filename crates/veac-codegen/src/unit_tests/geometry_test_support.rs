use veac_plan::EffectiveVisualProperties;

pub(crate) fn overlay_position(
    visual: &EffectiveVisualProperties,
    local_clock: &str,
    pivot_x: f64,
    pivot_y: f64,
) -> (String, String) {
    super::position(
        visual,
        local_clock,
        pivot_x,
        pivot_y,
        super::CoordinateSpace {
            width: "W",
            height: "H",
            source_width: "w",
            source_height: "h",
        },
    )
}
