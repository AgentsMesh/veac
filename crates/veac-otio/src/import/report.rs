use std::collections::BTreeMap;

use serde_json::Value;

use crate::OtioLossReport;

pub(super) fn extras(report: &mut OtioLossReport, pointer: &str, extra: &BTreeMap<String, Value>) {
    for key in extra.keys() {
        report.push(
            pointer,
            key,
            "unknown OTIO field is not represented canonically",
            false,
        );
    }
}

pub(super) fn values(report: &mut OtioLossReport, pointer: &str, field: &str, values: &[Value]) {
    if !values.is_empty() {
        report.push(
            pointer,
            field,
            "OTIO markers or effects require an explicit VEAC mapping",
            false,
        );
    }
}
