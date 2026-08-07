mod contract;
mod reachability;

use crate::ResolvedRenderPlan;

use super::Validator;

impl Validator {
    pub(super) fn temporal(&mut self, plan: &ResolvedRenderPlan) {
        if let Err(errors) = veac_ir::validate_temporal_library(&plan.temporal) {
            for diagnostic in errors.into_diagnostics() {
                self.push(
                    &diagnostic.code,
                    format!("/temporal{}", diagnostic.pointer),
                    diagnostic.message,
                );
            }
        }
        for (ordered, pointer, label) in [
            (
                sorted(plan.temporal.bindings.iter().map(|value| value.id.as_str())),
                "/temporal/bindings",
                "bindings",
            ),
            (
                sorted(plan.temporal.programs.iter().map(|value| value.id.as_str())),
                "/temporal/programs",
                "programs",
            ),
            (
                sorted(
                    plan.temporal
                        .provenance
                        .iter()
                        .map(|value| value.id.as_str()),
                ),
                "/temporal/provenance",
                "provenance records",
            ),
        ] {
            if !ordered {
                self.push(
                    "PLAN_TEMPORAL_ORDER",
                    pointer,
                    format!("temporal {label} must be uniquely sorted by ID"),
                );
            }
        }
        contract::Contract::new(self, plan).validate();
    }
}

fn sorted<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let values = values.collect::<Vec<_>>();
    values.windows(2).all(|pair| pair[0] < pair[1])
}
