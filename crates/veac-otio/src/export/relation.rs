use veac_ir::{Project, Relation, Sequence};

use crate::OtioLossReport;

pub(super) fn select(
    project: &Project,
    sequence: &Sequence,
    losses: &mut OtioLossReport,
) -> Vec<Relation> {
    project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == sequence.id)
        .map(|relation| {
            losses.push(
                format!("/project/relations/{}", relation.id),
                "kind",
                "standard OTIO omits typed VEAC relation semantics",
                true,
            );
            relation.clone()
        })
        .collect()
}
