use veac_ir::{TemporalParameterId, TemporalType, TemporalValue};
use veac_lang::program::expression::CoreTemporalInputIdentity;
use veac_lang::program::ClipTemporalProperty;

use super::{sink_support, support};

#[test]
fn an_audio_only_clip_cannot_receive_a_visual_temporal_leaf() {
    let (_, audio_id) = sink_support::items();
    let expression = support::compile(
        "progress",
        [(
            "progress",
            CoreTemporalInputIdentity::Progress {
                item_id: audio_id.clone(),
            },
        )],
    );
    let leaf = support::leaf(
        audio_id,
        ClipTemporalProperty::VisualOpacity,
        expression,
        "missing_visual",
    );
    let diagnostic = support::failure(sink_support::SOURCE, &[leaf]);
    assert!(diagnostic
        .message
        .contains("EXECUTABLE_TEMPORAL_VISUAL_SINK"));
}

#[test]
fn canonical_library_validation_rejects_non_finite_host_parameters() {
    let ids = support::ids(support::SOLID_SOURCE);
    let parameter_id = TemporalParameterId::new("tpm_non_finite").unwrap();
    let expression = support::compile(
        "gain",
        [(
            "gain",
            CoreTemporalInputIdentity::Parameter {
                parameter_id: parameter_id.clone(),
                value_type: TemporalType::Scalar,
            },
        )],
    );
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "non_finite",
    )
    .bind_parameter(parameter_id, TemporalValue::Scalar { value: f64::NAN });
    let diagnostic = support::failure(support::SOLID_SOURCE, &[leaf]);
    assert!(diagnostic.message.contains("EXECUTABLE_TEMPORAL_LIBRARY"));
}
