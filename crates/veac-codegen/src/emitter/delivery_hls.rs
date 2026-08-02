mod arguments;
mod graph;

use std::path::PathBuf;

use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{AdaptivePackage, Deliverable};
use veac_plan::ResolvedRenderPlan;

use super::{
    output, BackendAction, BackendCommand, BackendOutput, BackendPackagePaths, BackendPhase,
    BackendProduct, BackendTask, CodegenErrors, EmitContext,
};

const ENTRYPOINT: &str = "master.m3u8";
const PLAYLIST_PATTERN: &str = "rendition-%v.m3u8";
const SEGMENT_PATTERN: &str = "segment-%v-%06d.ts";

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    package: &AdaptivePackage,
) -> Result<BackendTask, CodegenErrors> {
    let AdaptivePackage::Hls(settings) = package;
    let mut context = EmitContext::new_visual(
        plan,
        bindings,
        deliverable,
        veac_plan::canonical::AlphaMode::Opaque,
    )?;
    let sequence = context
        .plan
        .sequences
        .iter()
        .find(|value| value.id == context.plan.entry_sequence_id)
        .ok_or_else(|| CodegenErrors::one(super::error::missing_entry(context.plan)))?;
    let outputs = graph::build(&mut context, sequence, settings)?;
    let maps = maps(&outputs);
    let root = output::bound_path(deliverable, bindings)?;
    let inputs = context.input_routes.backend_inputs().to_vec();
    let (filter_graph, filter_contract) = context.filter_graph()?;
    let preparations = context.take_preparations(&inputs);
    let command = BackendCommand {
        preparations,
        inputs,
        filter_graph,
        filter_contract,
        maps,
        output_args: arguments::build(settings, sequence.duration),
        output_path: PathBuf::from(PLAYLIST_PATTERN),
    };
    Ok(BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase: BackendPhase::Single,
        product: BackendProduct::HlsVod,
        output: BackendOutput::Package {
            root,
            entrypoint: PathBuf::from(ENTRYPOINT),
            paths: BackendPackagePaths {
                playlist_pattern: PathBuf::from(PLAYLIST_PATTERN),
                segment_pattern: PathBuf::from(SEGMENT_PATTERN),
            },
        },
        action: BackendAction::Ffmpeg(command),
    })
}

fn maps(outputs: &graph::Outputs) -> Vec<String> {
    let mut maps = Vec::with_capacity(outputs.video.len() * 2);
    for (index, video) in outputs.video.iter().enumerate() {
        maps.push(format!("[{video}]"));
        if let Some(audio) = &outputs.audio {
            maps.push(format!("[{}]", audio[index]));
        }
    }
    maps
}
