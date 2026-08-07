use std::collections::BTreeSet;

use veac_ir::{ProjectEnvelope, TemporalProgramLibrary};

use crate::ResolvedSequence;

use super::{collect, TemporalIssue};

pub(crate) fn select(
    envelope: &ProjectEnvelope,
    sequences: &[ResolvedSequence],
) -> Result<TemporalProgramLibrary, Vec<TemporalIssue>> {
    let sinks = collect(sequences);
    let binding_ids = sinks
        .iter()
        .map(|sink| sink.binding_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut issues = Vec::new();
    let mut bindings = Vec::with_capacity(binding_ids.len());
    let mut program_ids = BTreeSet::new();
    let mut provenance_ids = BTreeSet::new();
    for id in binding_ids {
        let Some(binding) = envelope
            .temporal
            .bindings
            .iter()
            .find(|value| value.id.as_str() == id)
        else {
            issues.push(TemporalIssue::new(
                "PLAN_TEMPORAL_BINDING_MISSING",
                "/temporal/bindings".to_owned(),
                "reachable animation sink binding disappeared during planning",
            ));
            continue;
        };
        program_ids.insert(binding.program_id.as_str());
        provenance_ids.insert(binding.provenance_id.as_str());
        bindings.push(binding.clone());
    }
    let mut programs = Vec::with_capacity(program_ids.len());
    for id in program_ids {
        let Some(program) = envelope
            .temporal
            .programs
            .iter()
            .find(|value| value.id.as_str() == id)
        else {
            issues.push(TemporalIssue::new(
                "PLAN_TEMPORAL_PROGRAM_MISSING",
                "/temporal/programs".to_owned(),
                "reachable temporal program disappeared during planning",
            ));
            continue;
        };
        provenance_ids.insert(program.provenance_id.as_str());
        for provenance in program
            .nodes
            .iter()
            .filter_map(|node| node.provenance_id.as_ref())
        {
            provenance_ids.insert(provenance.as_str());
        }
        programs.push(program.clone());
    }
    let mut provenance = Vec::with_capacity(provenance_ids.len());
    for id in provenance_ids {
        let Some(value) = envelope
            .temporal
            .provenance
            .iter()
            .find(|value| value.id.as_str() == id)
        else {
            issues.push(TemporalIssue::new(
                "PLAN_TEMPORAL_PROVENANCE_MISSING",
                "/temporal/provenance".to_owned(),
                "reachable temporal provenance disappeared during planning",
            ));
            continue;
        };
        provenance.push(value.clone());
    }
    bindings.sort_by(|left, right| left.id.cmp(&right.id));
    programs.sort_by(|left, right| left.id.cmp(&right.id));
    provenance.sort_by(|left, right| left.id.cmp(&right.id));
    if issues.is_empty() {
        Ok(TemporalProgramLibrary {
            opset_version: envelope.temporal.opset_version,
            programs,
            bindings,
            provenance,
        })
    } else {
        Err(issues)
    }
}
