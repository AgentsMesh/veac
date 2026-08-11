use super::*;
use veac_plan::canonical::{TemporalBindingId, Vec2};

#[test]
fn temporal_scale_is_clamped_to_the_valid_render_budget() {
    let value = Animatable::Binding {
        binding_id: TemporalBindingId::new("tbd_scale_bounds").unwrap(),
    };
    assert_eq!(
        bounded(&value, "100".to_owned()),
        "clip((100)\\,0.000001\\,16)"
    );
    assert_eq!(
        bounded(
            &Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            "1".to_owned()
        ),
        "1"
    );
}
