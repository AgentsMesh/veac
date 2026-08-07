use std::error::Error;

use crate::*;

#[test]
fn typed_ids_accept_stable_ascii_ids() {
    macro_rules! assert_id {
        ($type:ty, $value:literal) => {{
            let id = <$type>::new($value).unwrap();
            assert_eq!(id.as_str(), $value);
            assert_eq!(id.to_string(), $value);
            assert!(id.is_valid());
        }};
    }
    assert_id!(ProjectId, "prj_a");
    assert_id!(MaterialId, "med_a");
    assert_id!(SequenceId, "seq_a");
    assert_id!(TrackId, "trk_a");
    assert_id!(ItemId, "itm_a");
    assert_id!(MulticamGroupId, "mcg_a");
    assert_id!(MulticamAngleId, "ang_a");
    assert_id!(EffectId, "fx_a");
    assert_id!(AudioProcessorId, "aud_a");
    assert_id!(EqBandId, "eqb_a");
    assert_id!(KeyframeId, "kf_a");
    assert_id!(RelationId, "rel_a");
    assert_id!(BusId, "bus_a");
    assert_id!(RenderConfigId, "out_a");
    assert_id!(OperationId, "op_a-2");
}

#[test]
fn typed_ids_reject_wrong_prefix_empty_invalid_and_long_suffixes() {
    let wrong = ProjectId::new("seq_wrong").unwrap_err();
    assert_eq!(wrong.expected_prefix(), "prj_");
    assert_eq!(wrong.value(), "seq_wrong");
    assert_eq!(wrong.to_string(), "invalid prj_ identifier \"seq_wrong\"");
    assert!(wrong.source().is_none());
    assert!(ProjectId::new("prj_").is_err());
    assert!(ProjectId::new("prj_-bad").is_err());
    assert!(ProjectId::new("prj_bad.dot").is_err());
    assert!(RelationId::new("transition_intro").is_err());
    assert!(BusId::new("dialogue").is_err());
    assert!(AudioProcessorId::new("eqb_wrong").is_err());
    assert!(EqBandId::new("aud_wrong").is_err());
    assert!(ProjectId::new(format!("prj_{}", "a".repeat(125))).is_err());

    let invalid: ProjectId = serde_json::from_str("\"invalid\"").unwrap();
    assert!(!invalid.is_valid());
}

#[test]
fn typed_ids_construct_fixed_digest_identities_infallibly() {
    macro_rules! assert_digest {
        ($type:ty, $prefix:literal) => {{
            let id = <$type>::from_digest([0xab; 32]);
            assert_eq!(id.as_str(), format!("{}{}", $prefix, "ab".repeat(32)));
            assert!(id.is_valid());
        }};
    }
    assert_digest!(ProjectId, "prj_");
    assert_digest!(MaterialId, "med_");
    assert_digest!(SequenceId, "seq_");
    assert_digest!(TrackId, "trk_");
    assert_digest!(ItemId, "itm_");
    assert_digest!(ApplyId, "apl_");
    assert_digest!(ApplyStageId, "aps_");
    assert_digest!(MulticamGroupId, "mcg_");
    assert_digest!(MulticamAngleId, "ang_");
    assert_digest!(EffectId, "fx_");
    assert_digest!(AudioProcessorId, "aud_");
    assert_digest!(EqBandId, "eqb_");
    assert_digest!(KeyframeId, "kf_");
    assert_digest!(RenderConfigId, "out_");
    assert_digest!(DeliverableId, "dlv_");
    assert_digest!(HlsRenditionId, "rnd_");
    assert_digest!(OperationId, "op_");
    assert_digest!(AnnotationId, "ann_");
    assert_digest!(RelationId, "rel_");
    assert_digest!(BusId, "bus_");
}

#[test]
fn rational_time_arithmetic_and_order_are_exact() {
    let half = RationalTime::new(300, 600).unwrap();
    let same_half = RationalTime::new(1, 2).unwrap();
    assert_eq!(
        half.partial_cmp(&same_half),
        Some(std::cmp::Ordering::Equal)
    );
    assert_eq!(
        half.checked_add(half).unwrap(),
        RationalTime::new(600, 600).unwrap()
    );
    assert_eq!(RationalTime::zero(600).unwrap().value, 0);
    assert!(half.is_valid());
    assert_eq!(RationalTime::new(0, 0), Err(TimeError::ZeroTimescale));

    let invalid: RationalTime = serde_json::from_str(r#"{"value":0,"timescale":0}"#).unwrap();
    assert_eq!(invalid.partial_cmp(&half), None);
    assert_eq!(half.partial_cmp(&invalid), None);
    assert_eq!(
        half.checked_add(same_half),
        Err(TimeError::MismatchedTimescale)
    );
    let maximum: RationalTime =
        serde_json::from_str(&format!(r#"{{"value":{},"timescale":600}}"#, i64::MAX)).unwrap();
    assert_eq!(
        maximum.checked_add(RationalTime::new(1, 600).unwrap()),
        Err(TimeError::Overflow)
    );
}

#[test]
fn ranges_and_ratios_enforce_their_structural_contract() {
    let start = RationalTime::new(0, 600).unwrap();
    let duration = RationalTime::new(30, 600).unwrap();
    let range = TimeRange::new(start, duration).unwrap();
    assert_eq!(range.end().unwrap(), RationalTime::new(30, 600).unwrap());
    assert_eq!(
        TimeRange::new(start, RationalTime::new(0, 600).unwrap()),
        Err(TimeError::NonPositiveDuration)
    );
    assert_eq!(
        TimeRange::new(start, RationalTime::new(1, 1).unwrap()),
        Err(TimeError::MismatchedTimescale)
    );
    assert!(Rational::new(30, 1).unwrap().is_positive());
    assert_eq!(Rational::new(60, 2).unwrap(), Rational::new(30, 1).unwrap());
    assert!(!Rational::new(0, 1).unwrap().is_positive());
    assert_eq!(Rational::new(1, 0), Err(TimeError::ZeroTimescale));
    assert_eq!(
        TimeError::ZeroTimescale.to_string(),
        "timescale must be greater than zero"
    );
    assert_eq!(
        TimeError::MismatchedTimescale.to_string(),
        "time values use different timescales"
    );
    assert_eq!(
        TimeError::NonPositiveDuration.to_string(),
        "duration must be greater than zero"
    );
    assert_eq!(
        TimeError::Overflow.to_string(),
        "time arithmetic overflowed"
    );
    assert_eq!(
        TimeError::UnsafeInteger.to_string(),
        "integer is outside the exact I-JSON range"
    );
    assert_eq!(
        RationalTime::new(i64::MAX, 600),
        Err(TimeError::UnsafeInteger)
    );
    assert!(TimeError::Overflow.source().is_none());
}
