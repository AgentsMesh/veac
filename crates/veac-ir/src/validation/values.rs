use std::collections::BTreeMap;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn time_range(
        &mut self,
        range: TimeRange,
        timebase: u32,
        code: &str,
        path: &str,
        object_id: &str,
    ) {
        self.time(range.start, timebase, false, code, path, object_id);
        self.time(range.duration, timebase, true, code, path, object_id);
        if range.start.timescale != range.duration.timescale || range.end().is_err() {
            self.value_error(code, path, object_id);
        }
    }

    pub(super) fn time(
        &mut self,
        time: RationalTime,
        timebase: u32,
        positive: bool,
        code: &str,
        path: &str,
        object_id: &str,
    ) {
        if !time.is_valid()
            || time.timescale != timebase
            || (positive && time.value <= 0)
            || (!positive && time.value < 0)
        {
            self.value_error(code, path, object_id);
        }
    }

    pub(super) fn intrinsic_time(
        &mut self,
        time: RationalTime,
        positive: bool,
        code: &str,
        path: &str,
        object_id: &str,
    ) {
        if !time.is_valid() || (positive && time.value <= 0) || (!positive && time.value < 0) {
            self.value_error(code, path, object_id);
        }
    }

    pub(super) fn length(
        &mut self,
        length: Length,
        positive: bool,
        code: &str,
        path: &str,
        object_id: &str,
    ) {
        if !length.value.is_finite() || (positive && length.value <= 0.0) {
            self.value_error(code, path, object_id);
        }
    }

    pub(super) fn finite_vec(
        &mut self,
        value: Vec2,
        nonnegative: bool,
        code: &str,
        path: &str,
        object_id: &str,
    ) {
        if !value.x.is_finite()
            || !value.y.is_finite()
            || (nonnegative && (value.x < 0.0 || value.y < 0.0))
        {
            self.value_error(code, path, object_id);
        }
    }

    pub(super) fn metadata(
        &mut self,
        metadata: &BTreeMap<String, serde_json::Value>,
        path: &str,
        object_id: &str,
    ) {
        if !metadata.values().all(json_value_is_ijson) {
            self.value_error("METADATA_NUMBER", path, object_id);
        }
    }
}

pub(super) fn is_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && !matches!(value, "." | "..")
        && !value
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
}

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn json_value_is_ijson(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .map(crate::time::safe_i64)
            .or_else(|| number.as_u64().map(crate::time::safe_u64))
            .unwrap_or(true),
        serde_json::Value::Array(values) => values.iter().all(json_value_is_ijson),
        serde_json::Value::Object(values) => values.values().all(json_value_is_ijson),
        _ => true,
    }
}
