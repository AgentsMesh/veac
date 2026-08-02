use crate::arguments::OtioCommand;
use crate::unit_tests::support::{canonical_project, GENERATED_SOURCE};

#[test]
fn otio_commands_reject_bad_ids_stale_extensions_and_input_aliases() {
    let temp = tempfile::tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let output = temp.path().join("timeline.otio");
    let loss = temp.path().join("timeline.loss.json");
    let export = |sequence, output, loss_report| OtioCommand::Export {
        project: project.clone(),
        sequence,
        output,
        loss_report,
        allow_lossy: true,
    };
    for sequence in ["bad id", "seq_missing"] {
        assert!(crate::commands::otio(export(
            Some(sequence.to_owned()),
            Some(output.clone()),
            Some(loss.clone())
        ))
        .is_err());
    }
    assert!(
        crate::commands::otio(export(None, Some(project.clone()), Some(loss.clone()))).is_err()
    );
    assert!(crate::commands::otio(export(None, Some(output.clone()), Some("-".into()))).is_err());

    let envelope = crate::canonical::load(&project).unwrap();
    let mut timeline = veac_otio::export_sequence(&envelope, &envelope.project.entry_sequence_id)
        .unwrap()
        .timeline;
    std::fs::write(&output, veac_otio::canonical_otio_json(&timeline).unwrap()).unwrap();
    let target = super::target_project(&temp, &project);
    let propose = |operation_id, destination| OtioCommand::Propose {
        project: target.clone(),
        timeline: output.clone(),
        bindings: None,
        operation_id,
        output: destination,
        loss_report: None,
        allow_lossy: false,
    };
    assert!(crate::commands::otio(propose("bad id".to_owned(), None)).is_err());
    assert!(crate::commands::otio(propose("op_alias".to_owned(), Some(target.clone()))).is_err());

    timeline.name.push_str(" stale");
    std::fs::write(&output, veac_otio::canonical_otio_json(&timeline).unwrap()).unwrap();
    assert!(crate::commands::otio(propose("op_stale".to_owned(), None)).is_err());
}
