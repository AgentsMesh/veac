use veac_codegen::emitter::{BackendAction, BackendBundle, BackendCommand};
use veac_plan::canonical::{Deliverable, DeliverableId, DeliverableKind, DeliverableTarget};

pub(super) fn file(id: &str, name: &str, kind: DeliverableKind) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: name.to_owned(),
        },
        kind,
    }
}

pub(super) fn ffmpeg<'a>(bundle: &'a BackendBundle, id: &str) -> &'a BackendCommand {
    let task = bundle
        .tasks()
        .iter()
        .find(|task| task.deliverable_id.as_str() == id)
        .unwrap_or_else(|| panic!("missing task {id}"));
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!("task {id} must use FFmpeg")
    };
    command
}

pub(super) fn pair(arguments: &[String], name: &str, value: &str) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == name && pair[1] == value)
}
