mod finishing;
mod media;
mod relations;
mod text;
mod visual;

pub use finishing::*;
pub use media::*;
pub use relations::*;
pub use text::*;
pub use visual::*;

use std::collections::BTreeMap;
use std::path::PathBuf;

use veac_artifact::ExecutionBindings;
use veac_codegen::emitter::{
    emit_all, BackendAction, BackendCommand, BackendProduct, CodegenErrors,
};
use veac_plan::canonical::*;
use veac_plan::{resolve_one, PlanInputId, ResolvedInputKind, ResolvedRenderPlan};

pub fn fixture() -> ProjectEnvelope {
    let mut envelope: ProjectEnvelope = serde_json::from_str(include_str!(
        "../../../../veac-ir/tests/fixtures/minimal-project.json"
    ))
    .expect("canonical fixture");
    let identity = identity('a');
    envelope.project.materials[0].identity = Some(identity.clone());
    envelope.project.materials[0].probe = Some(probe(identity));
    envelope
}

pub fn resolved(envelope: &ProjectEnvelope) -> ResolvedRenderPlan {
    resolve_one(envelope, &RenderConfigId::new("out_main").unwrap()).expect("resolved plan")
}

pub fn bindings(plan: &ResolvedRenderPlan) -> ExecutionBindings {
    let mut bindings = input_bindings(plan);
    bind_outputs(plan, &mut bindings);
    bindings
}

pub fn input_bindings(plan: &ResolvedRenderPlan) -> ExecutionBindings {
    let paths = plan
        .inputs
        .iter()
        .map(|input| (input.id.clone(), default_input_path(input)))
        .collect::<BTreeMap<_, _>>();
    ExecutionBindings::from_originals(plan, &paths).unwrap()
}

pub fn bindings_with_original(
    plan: &ResolvedRenderPlan,
    id: &PlanInputId,
    path: PathBuf,
) -> ExecutionBindings {
    let paths = plan
        .inputs
        .iter()
        .map(|input| {
            let value = if input.id == *id {
                path.clone()
            } else {
                default_input_path(input)
            };
            (input.id.clone(), value)
        })
        .collect::<BTreeMap<_, _>>();
    let mut bindings = ExecutionBindings::from_originals(plan, &paths).unwrap();
    bind_outputs(plan, &mut bindings);
    bindings
}

pub fn output_bindings(plan: &ResolvedRenderPlan) -> ExecutionBindings {
    let mut bindings = ExecutionBindings::default();
    bind_outputs(plan, &mut bindings);
    bindings
}

fn bind_outputs(plan: &ResolvedRenderPlan, bindings: &mut ExecutionBindings) {
    for deliverable in &plan.output.deliverables {
        bindings
            .bind_output(
                deliverable.id.clone(),
                PathBuf::from("/tmp").join(target_name(deliverable)),
            )
            .unwrap();
    }
}

pub fn target_name(deliverable: &Deliverable) -> &str {
    match &deliverable.target {
        DeliverableTarget::File { name } => name,
        DeliverableTarget::ImageSequence { pattern } => pattern,
        DeliverableTarget::Package { name } => name,
    }
}

fn default_input_path(input: &veac_plan::ResolvedInput) -> PathBuf {
    match &input.kind {
        ResolvedInputKind::Font { .. } => test_font_path(),
        ResolvedInputKind::Resource { material_kind } => lut_fixture(*material_kind),
        ResolvedInputKind::Media { .. } => PathBuf::from(format!("/tmp/{}.bin", input.id)),
    }
}

pub fn emit_video_command(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
) -> Result<BackendCommand, CodegenErrors> {
    let bundle = emit_all(plan, bindings)?;
    let commands = bundle
        .tasks()
        .iter()
        .filter_map(|task| match (&task.action, task.product) {
            (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster) => Some(command.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [command] = commands.as_slice() else {
        panic!("test plan must emit exactly one video master")
    };
    Ok(command.clone())
}

pub fn emit_video_command_for(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    id: &DeliverableId,
) -> Result<BackendCommand, CodegenErrors> {
    let bundle = emit_all(plan, bindings)?;
    let commands = bundle
        .tasks()
        .iter()
        .filter_map(
            |task| match (&task.action, task.product, &task.deliverable_id) {
                (BackendAction::Ffmpeg(command), BackendProduct::VideoMaster, value)
                    if value == id =>
                {
                    Some(command.clone())
                }
                _ => None,
            },
        )
        .collect::<Vec<_>>();
    let [command] = commands.as_slice() else {
        panic!("deliverable {id} must emit exactly one video master")
    };
    Ok(command.clone())
}

pub fn assert_rgb_plane_output(graph: &str, prefix: &str) {
    let output = format!("[{prefix}");
    assert!(
        graph
            .split(';')
            .any(|node| node.contains("mergeplanes=format=gbrp16le") && node.contains(&output)),
        "missing explicit RGB-plane output {prefix}: {graph}"
    );
}

pub fn test_font_path() -> PathBuf {
    [
        "/System/Library/Fonts/SFNSMono.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file())
    .expect("a supported system test font")
}

pub fn alternate_test_font_path() -> PathBuf {
    [
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    ]
    .iter()
    .map(PathBuf::from)
    .find(|path| path.is_file() && path.parent() != test_font_path().parent())
    .expect("a second system test font in another directory")
}

pub fn set_input_identity(plan: &mut ResolvedRenderPlan, id: &PlanInputId, path: &std::path::Path) {
    plan.inputs
        .iter_mut()
        .find(|input| input.id == *id)
        .expect("test input")
        .observed_identity = file_identity(path);
}
