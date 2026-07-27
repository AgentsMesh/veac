use std::path::PathBuf;

use super::*;
use crate::emitter::{BackendCommand, BackendOutput, BackendPhase, BackendProduct};
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

#[test]
fn explicit_hardware_arguments_become_typed_requirements() {
    let plan = resolved(&fixture());
    let task = BackendTask {
        deliverable_id: plan.output.deliverables[0].id.clone(),
        phase: BackendPhase::Single,
        product: BackendProduct::VideoMaster,
        output: BackendOutput::File(PathBuf::from("master.mp4")),
        action: BackendAction::Ffmpeg(BackendCommand {
            inputs: vec![],
            filter_graph: None,
            filter_contract: None,
            maps: vec![],
            output_args: vec![
                "-hwaccel".to_owned(),
                "videotoolbox".to_owned(),
                "-init_hw_device".to_owned(),
                "videotoolbox=device:0".to_owned(),
            ],
            output_path: PathBuf::from("master.mp4"),
        }),
    };
    let values = collect(&plan, &ExecutionBindings::default(), &[task]).unwrap();
    for kind in [
        BackendCapabilityKind::HardwareBackend,
        BackendCapabilityKind::HardwareDevice,
    ] {
        assert!(values
            .iter()
            .any(|value| value.kind() == kind && value.name() == "videotoolbox"));
    }
}
