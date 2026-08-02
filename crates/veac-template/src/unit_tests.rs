#[path = "../tests/binding_errors.rs"]
mod binding_errors;
#[path = "../tests/contract_errors.rs"]
mod contract_errors;
#[path = "../tests/fill_modes.rs"]
mod fill_modes;
#[path = "../tests/frame_geometry.rs"]
mod frame_geometry;
#[path = "../tests/identity_errors.rs"]
mod identity_errors;
#[path = "../tests/media_errors.rs"]
mod media_errors;
#[path = "../tests/request_contract.rs"]
mod request_contract;
#[path = "../tests/text_and_atomic.rs"]
mod text_and_atomic;

#[test]
fn private_timing_rejects_nonpositive_and_unrepresentable_rates() {
    assert_eq!(
        crate::timing::ratio(0, 1).unwrap_err().kind,
        crate::TemplateErrorKind::InexactTime
    );
    assert_eq!(
        crate::timing::ratio(1, 0).unwrap_err().kind,
        crate::TemplateErrorKind::InexactTime
    );
    assert_eq!(
        crate::timing::ratio(1, i64::from(u32::MAX) + 1)
            .unwrap_err()
            .kind,
        crate::TemplateErrorKind::InexactTime
    );
}
