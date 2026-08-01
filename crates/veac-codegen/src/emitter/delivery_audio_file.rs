use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AudioFile, AudioFileEncoding, Deliverable, Mp3Encoding};
use veac_plan::ResolvedRenderPlan;

use super::{
    audio, output, time, BackendAction, BackendCommand, BackendOutput, BackendPhase,
    BackendProduct, BackendTask, CodegenErrors, EmitContext,
};

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &AudioFile,
) -> Result<BackendTask, CodegenErrors> {
    let mut context = EmitContext::new_audio(plan, bindings, deliverable)?;
    let sequence = context
        .plan
        .sequences
        .iter()
        .find(|value| value.id == context.plan.entry_sequence_id)
        .ok_or_else(|| CodegenErrors::one(super::error::missing_entry(context.plan)))?;
    let AudioFileEncoding::Mp3(encoding) = &settings.encoding;
    let audio = audio::build_stem(
        &mut context,
        sequence,
        &settings.source,
        audio::AudioRenderSpec {
            sample_rate: encoding.sample_rate_hz,
            channels: encoding.channel_layout.count(),
        },
    )?;
    let path = output::bound_path(deliverable, bindings)?;
    let output_args = match &settings.encoding {
        AudioFileEncoding::Mp3(encoding) => mp3_arguments(encoding, sequence.duration),
    };
    let (filter_graph, filter_contract) = context.filter_graph()?;
    let command = BackendCommand {
        preparations: Vec::new(),
        inputs: context.input_routes.backend_inputs().to_vec(),
        filter_graph,
        filter_contract,
        maps: vec![format!("[{audio}]")],
        output_args,
        output_path: path.clone(),
    };
    Ok(BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase: BackendPhase::Single,
        product: BackendProduct::AudioFile,
        output: BackendOutput::File(path),
        action: BackendAction::Ffmpeg(command),
    })
}

fn mp3_arguments(
    settings: &Mp3Encoding,
    duration: veac_plan::canonical::RationalTime,
) -> Vec<String> {
    vec![
        "-vn".into(),
        "-c:a".into(),
        "libmp3lame".into(),
        "-b:a".into(),
        settings.bitrate_bps.to_string(),
        "-ar".into(),
        settings.sample_rate_hz.to_string(),
        "-ac".into(),
        settings.channel_layout.count().to_string(),
        "-f".into(),
        "mp3".into(),
        "-t".into(),
        time::seconds(duration),
    ]
}
