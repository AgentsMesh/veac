use veac_evidence::{DecodeObservation, LayerOrderObservation, ObservationSet, RationalTime};

pub fn observations() -> ObservationSet {
    let mut value = ObservationSet::default();
    value.frames.insert("under".into(), super::underlay());
    value.frames.insert("overlay".into(), super::overlay());
    value.frames.insert("actual".into(), super::actual());
    value.frames.insert("reveal1".into(), super::reveal(1));
    value.frames.insert("reveal2".into(), super::reveal(2));
    value.frames.insert("motion0".into(), super::motion(0));
    value.frames.insert("motion1".into(), super::motion(12));
    value.frames.insert("motion2".into(), super::motion(16));
    value.decodes.insert(
        "final".into(),
        DecodeObservation {
            complete: true,
            decoded_frames: 16,
            last_pts: Some(RationalTime {
                value: 15,
                timescale: 30,
            }),
            errors: vec![],
        },
    );
    value.layer_orders.insert(
        "layer".into(),
        LayerOrderObservation {
            upper_entity: "copy".into(),
            lower_entity: "picture".into(),
            upper_order: 2,
            lower_order: 1,
            coactive: true,
        },
    );
    value
}
