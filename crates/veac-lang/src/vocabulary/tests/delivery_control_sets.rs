use std::collections::BTreeSet;

use crate::vocabulary::control_uses::delivery as c;

use super::super::{language_spec, CanonicalRole as R, ControlUse, GrammarPosition as P};
use super::legacy_surface::{assert_control_absent, assert_control_contract};

mod extended;
mod fields;
mod shared;

#[test]
fn delivery_controls_have_exact_owner_roles_and_complete_coverage() {
    let mut expected = Vec::new();
    for (position, controls) in fields::groups().into_iter().chain(extended::groups()) {
        check_group(&mut expected, position, R::FieldIntroducer, controls);
    }
    for (control, position, role) in special_uses() {
        check_group(&mut expected, position, role, &[control]);
    }
    assert_eq!(expected.len(), 81);
    assert_eq!(as_set(&expected), as_set(c::ALL));
}

fn special_uses() -> [(ControlUse, P, R); 9] {
    [
        (
            c::ARTIFACT_MEMBER,
            P::DeliveryMember,
            R::DeclarationIntroducer,
        ),
        (
            c::RASTER_CANVAS_BY,
            P::DeliveryRasterCanvasSeparator,
            R::InfixSeparator,
        ),
        (
            c::CAPTION_TRACKS_KIND,
            P::CaptionSidecarSourceKind,
            R::KindDiscriminator,
        ),
        (
            c::CAPTION_TRACK_REFERENCE,
            P::CaptionTracksMember,
            R::ReferenceKind,
        ),
        (
            c::FRAME_CONTAINING_CLAUSE,
            P::OutputFrameSelectionClause,
            R::ClauseIntroducer,
        ),
        (
            c::SCOPE_CANVAS_BY,
            P::ScopeCanvasSeparator,
            R::InfixSeparator,
        ),
        (
            c::IMAGE_NUMBERING_FROM,
            P::ImageNumberingFromClause,
            R::ClauseIntroducer,
        ),
        (
            c::HLS_RENDITION_MEMBER,
            P::HlsPackageMember,
            R::DeclarationIntroducer,
        ),
        (
            c::HLS_RENDITION_CANVAS_BY,
            P::HlsRenditionCanvasSeparator,
            R::InfixSeparator,
        ),
    ]
}

fn check_group(expected: &mut Vec<ControlUse>, position: P, role: R, controls: &[ControlUse]) {
    let spec = language_spec();
    for control in controls {
        assert_control_contract(*control, position, role);
        assert_control_absent(&spec.vocabulary, *control);
        expected.push(*control);
    }
}

fn as_set(values: &[ControlUse]) -> BTreeSet<ControlUse> {
    values.iter().copied().collect()
}
