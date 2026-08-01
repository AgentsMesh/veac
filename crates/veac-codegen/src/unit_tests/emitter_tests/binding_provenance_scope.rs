use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::ResolvedInputKind;

use super::support::{fixture, output_bindings, resolved, test_font_path, text_fixture};

#[test]
fn consumed_original_binding_must_match_source_identity_stream_and_clock() {
    let plan = resolved(&fixture());

    let mut identity = plan.inputs[0].clone();
    identity.observed_identity.digest = "b".repeat(64);
    assert_forged(&plan, identity, "logical source identity");

    let mut stream = plan.inputs[0].clone();
    stream.video.as_mut().unwrap().selection.global_index += 1;
    assert_forged(&plan, stream, "logical stream");

    let mut clock = plan.inputs[0].clone();
    clock
        .probe
        .as_mut()
        .unwrap()
        .container_duration
        .as_mut()
        .unwrap()
        .timescale = 1_000;
    assert_forged(&plan, clock, "source clock");
}

#[test]
fn unused_forged_binding_never_enters_caption_task_authority() {
    let mut plan = resolved(&text_fixture(true));
    let caption_track = plan.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .id
        .clone();
    plan.output.deliverables = vec![Deliverable {
        id: DeliverableId::new("dlv_provenance_caption").unwrap(),
        target: DeliverableTarget::File {
            name: "provenance.srt".to_owned(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Srt,
            track_ids: vec![caption_track],
        }),
    }];
    plan.output.raster = None;
    let mut bindings = output_bindings(&plan);
    let mut forged = plan
        .inputs
        .iter()
        .find(|input| matches!(input.kind, ResolvedInputKind::Media { .. }))
        .unwrap()
        .clone();
    forged.observed_identity.digest = "c".repeat(64);
    bindings
        .bind_original(&forged, "/tmp/unused-forged.mov".into())
        .unwrap();

    let bundle = emit_all(&plan, &bindings).unwrap();

    assert!(bundle.protected_resources().is_empty());
}

#[test]
fn consumed_font_binding_cannot_substitute_a_same_id_resource() {
    let plan = resolved(&text_fixture(true));
    let mut bindings = output_bindings(&plan);
    for input in &plan.inputs {
        match &input.kind {
            ResolvedInputKind::Media { .. } => bindings
                .bind_original(input, format!("/tmp/{}.mov", input.id).into())
                .unwrap(),
            ResolvedInputKind::Font { .. } => {
                let mut forged = input.clone();
                forged.observed_identity.digest = "d".repeat(64);
                bindings.bind_original(&forged, test_font_path()).unwrap();
            }
            ResolvedInputKind::Resource { .. } => unreachable!(),
        }
    }

    let error = emit_all(&plan, &bindings).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "RESOURCE_BINDING_INVALID");
    assert!(error.diagnostics()[0]
        .message
        .contains("logical source identity"));
}

fn assert_forged(
    plan: &veac_plan::ResolvedRenderPlan,
    forged: veac_plan::ResolvedInput,
    marker: &str,
) {
    let mut bindings = output_bindings(plan);
    bindings
        .bind_original(&forged, "/tmp/forged-source.mov".into())
        .unwrap();
    let error = emit_all(plan, &bindings).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "RESOURCE_BINDING_INVALID");
    assert!(error.diagnostics()[0].message.contains(marker));
}
