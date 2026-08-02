use veac_ir::{ItemId, SequenceId, TrackId};

use crate::*;

use super::support::*;

#[test]
fn asr_rejects_a_missing_target_sequence() {
    let (request, response) = exchange(Capability::Asr);
    let mut context = asr_context();
    let ApplicationContext::AsrCaptions(value) = &mut context else {
        unreachable!()
    };
    value.sequence_id = SequenceId::new("seq_missing").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

#[test]
fn matte_rejects_a_missing_or_disabled_visual_target() {
    let (request, response, mut missing) = matte_fixture();
    matte(&mut missing).target_clip_id = ItemId::new("itm_missing").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &missing));

    let (request, response, context) = matte_fixture();
    let mut disabled = project();
    disabled.project.sequences[0].tracks[3].clips[0].enabled = false;
    assert_invalid(propose_edit(&disabled, &request, &response, &context));
}

#[test]
fn matte_rejects_a_missing_output_track() {
    let (request, response, mut context) = matte_fixture();
    matte(&mut context).matte.track_id = TrackId::new("trk_missing").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

#[test]
fn retouch_rejects_a_missing_apply_sequence() {
    let (request, response) = exchange(Capability::Retouch);
    let ProviderOutput::Retouch(result) = &response.output else {
        unreachable!()
    };
    let mut context = retouch_context(result);
    let ApplicationContext::Retouch(value) = &mut context else {
        unreachable!()
    };
    value.sequence_id = SequenceId::new("seq_missing").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

fn matte_fixture() -> (
    ProviderRequestEnvelope,
    ProviderResponseEnvelope,
    ApplicationContext,
) {
    let (request, response) = exchange(Capability::Matte);
    let ProviderOutput::Matte(result) = &response.output else {
        unreachable!()
    };
    let context = matte_context(Capability::Matte, &result.matte);
    (request, response, context)
}

fn matte(value: &mut ApplicationContext) -> &mut MatteApplication {
    let ApplicationContext::Matte(value) = value else {
        unreachable!()
    };
    value
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
