use veac_artifact::ExecutionBindings;
use veac_plan::ResolvedRenderPlan;

use super::super::{
    BackendAction, BackendCapabilityKind, BackendRequirement, BackendTask, CodegenErrors,
};

mod filter;
mod input;

pub(super) fn collect(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    tasks: &[BackendTask],
) -> Result<Vec<BackendRequirement>, CodegenErrors> {
    let mut requirements = Vec::new();
    for task in tasks {
        let BackendAction::Ffmpeg(command) = &task.action else {
            continue;
        };
        for pair in command.output_args.windows(2) {
            match pair[0].as_str() {
                "-c" | "-c:v" | "-c:a" if pair[1] != "copy" => push(
                    &mut requirements,
                    task,
                    BackendCapabilityKind::Encoder,
                    &pair[1],
                ),
                "-f" => push(
                    &mut requirements,
                    task,
                    BackendCapabilityKind::Muxer,
                    &pair[1],
                ),
                "-hwaccel" => push(
                    &mut requirements,
                    task,
                    BackendCapabilityKind::HardwareBackend,
                    &pair[1],
                ),
                "-init_hw_device" => push(
                    &mut requirements,
                    task,
                    BackendCapabilityKind::HardwareDevice,
                    device_type(&pair[1]),
                ),
                _ => {}
            }
        }
        if let Some(graph) = &command.filter_graph {
            for name in filter::names(graph) {
                push(
                    &mut requirements,
                    task,
                    BackendCapabilityKind::Filter,
                    &name,
                );
            }
        }
        for (kind, name) in input::capabilities(plan, bindings, command)? {
            push(&mut requirements, task, kind, &name);
        }
    }
    Ok(requirements)
}

fn push(
    values: &mut Vec<BackendRequirement>,
    task: &BackendTask,
    kind: BackendCapabilityKind,
    name: &str,
) {
    if values.iter().any(|value| {
        value.kind() == kind
            && value.deliverable_id() == &task.deliverable_id
            && value.name() == name
    }) {
        return;
    }
    let deliverable_id = task.deliverable_id.clone();
    let name = name.to_owned();
    values.push(match kind {
        BackendCapabilityKind::Encoder => BackendRequirement::Encoder {
            deliverable_id,
            name,
        },
        BackendCapabilityKind::Decoder => BackendRequirement::Decoder {
            deliverable_id,
            name,
        },
        BackendCapabilityKind::Muxer => BackendRequirement::Muxer {
            deliverable_id,
            name,
        },
        BackendCapabilityKind::Demuxer => BackendRequirement::Demuxer {
            deliverable_id,
            name,
        },
        BackendCapabilityKind::Filter => BackendRequirement::Filter {
            deliverable_id,
            name,
        },
        BackendCapabilityKind::HardwareBackend => BackendRequirement::HardwareBackend {
            deliverable_id,
            name,
        },
        BackendCapabilityKind::HardwareDevice => BackendRequirement::HardwareDevice {
            deliverable_id,
            name,
        },
    });
}

fn device_type(value: &str) -> &str {
    value.split(['=', ':']).next().unwrap_or(value)
}

#[cfg(test)]
#[path = "requirements/tests.rs"]
mod tests;
