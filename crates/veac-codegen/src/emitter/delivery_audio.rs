use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AudioStemFormat, AudioStemOutput, Deliverable};
use veac_plan::ResolvedRenderPlan;

use super::{
    audio, output, time, BackendAction, BackendCommand, BackendOutput, BackendPhase,
    BackendProduct, BackendTask, CodegenErrors, EmitContext,
};

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &AudioStemOutput,
) -> Result<BackendTask, CodegenErrors> {
    let mut context = EmitContext::new_audio(plan, bindings, deliverable)?;
    let Some(sequence) = context
        .plan
        .sequences
        .iter()
        .find(|value| value.id == context.plan.entry_sequence_id)
    else {
        return Err(CodegenErrors::one(super::error::missing_entry(
            context.plan,
        )));
    };
    let audio = audio::build_stem(
        &mut context,
        sequence,
        &settings.source,
        audio::AudioRenderSpec::from(&settings.audio),
    )?;
    let inputs = context.input_routes.backend_inputs().to_vec();
    let path = output::bound_path(deliverable, bindings)?;
    let output_args = vec![
        "-vn".to_owned(),
        "-c:a".to_owned(),
        output::audio_encoder(settings.audio.codec).to_owned(),
        "-ar".to_owned(),
        settings.audio.sample_rate.to_string(),
        "-ac".to_owned(),
        settings.audio.channels.to_string(),
        "-f".to_owned(),
        container(settings.format).to_owned(),
        "-t".to_owned(),
        time::seconds(sequence.duration),
    ];
    let (filter_graph, filter_contract) = context.filter_graph()?;
    let command = BackendCommand {
        preparations: Vec::new(),
        inputs,
        filter_graph,
        filter_contract,
        maps: vec![format!("[{audio}]")],
        output_args,
        output_path: path.clone(),
    };
    Ok(BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase: BackendPhase::Single,
        product: BackendProduct::AudioStem,
        output: BackendOutput::File(path),
        action: BackendAction::Ffmpeg(command),
    })
}

fn container(format: AudioStemFormat) -> &'static str {
    match format {
        AudioStemFormat::Wav => "wav",
        AudioStemFormat::Flac => "flac",
    }
}
