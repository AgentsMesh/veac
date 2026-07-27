use veac_ir::TrackId;

use crate::*;

use super::support::*;

#[test]
fn generated_audio_requires_normalized_probe_facts() {
    let (request, response, mut context) = fixture();
    output(&mut context).material.material.probe = None;
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

#[test]
fn generated_audio_requires_an_exact_selected_audio_stream() {
    let (request, response, mut missing) = fixture();
    probe(&mut missing).selected_audio_stream = None;
    assert_invalid(propose_edit(&project(), &request, &response, &missing));

    let (request, response, mut inconsistent) = fixture();
    probe(&mut inconsistent).streams[0].global_index = 1;
    assert_invalid(propose_edit(&project(), &request, &response, &inconsistent));
}

#[test]
fn generated_audio_requires_exact_available_duration() {
    let (request, response, mut context) = fixture();
    let probe = probe(&mut context);
    probe.container_duration = None;
    probe.streams[0].duration = None;
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

#[test]
fn generated_audio_rejects_a_missing_target_track() {
    let (request, response, mut context) = fixture();
    output(&mut context).track_id = TrackId::new("trk_missing").unwrap();
    assert_invalid(propose_edit(&project(), &request, &response, &context));
}

fn fixture() -> (
    ProviderRequestEnvelope,
    ProviderResponseEnvelope,
    ApplicationContext,
) {
    let (request, response) = exchange(Capability::TextToSpeech);
    let ProviderOutput::TextToSpeech(result) = &response.output else {
        unreachable!()
    };
    let context = tts_context(&result.audio);
    (request, response, context)
}

fn output(value: &mut ApplicationContext) -> &mut AudioClipInsertion {
    let ApplicationContext::TextToSpeech(value) = value else {
        unreachable!()
    };
    &mut value.output
}

fn probe(value: &mut ApplicationContext) -> &mut veac_ir::MediaProbeSnapshot {
    output(value).material.material.probe.as_mut().unwrap()
}

fn assert_invalid(value: ProviderResult<ProviderEditProposal>) {
    assert_eq!(value.unwrap_err().kind, ProviderErrorKind::InvalidContract);
}
