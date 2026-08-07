use crate::{OtioItem, OtioLossReport, OtioMediaReference, OtioTimeline, OtioTrack};

pub(super) fn timeline(value: &OtioTimeline, report: &mut OtioLossReport) {
    for (present, pointer, field, reason) in [
        (
            value.global_start_time.is_some(),
            "",
            "global_start_time",
            "VEAC sequence timelines start at canonical zero",
        ),
        (
            !value.metadata.is_empty(),
            "",
            "metadata",
            "OTIO timeline metadata is excluded; canonical authorship retains only a typed document digest",
        ),
        (
            value.tracks.source_range.is_some(),
            "/tracks",
            "source_range",
            "OTIO stack trimming has no VEAC sequence mapping",
        ),
        (
            !value.tracks.metadata.is_empty(),
            "/tracks",
            "metadata",
            "OTIO stack metadata is excluded from the closed canonical IR",
        ),
        (
            !value.tracks.enabled,
            "/tracks",
            "enabled",
            "VEAC has no sequence-stack enabled state",
        ),
    ] {
        if present {
            report.push(pointer, field, reason, false);
        }
    }
}

pub(super) fn annotations(value: &OtioTimeline, report: &mut OtioLossReport) {
    let reason = if value.metadata.contains_key(crate::export::extension::KEY) {
        "bound import does not trust a VEAC annotation extension after projection edits"
    } else {
        "third-party OTIO has no VEAC extension for typed annotations"
    };
    report.push("", "annotations", reason, false);
}

pub(super) fn track(value: &OtioTrack, pointer: &str, report: &mut OtioLossReport) {
    for (present, field, reason) in [
        (
            value.source_range.is_some(),
            "source_range",
            "OTIO track trimming has no VEAC track mapping",
        ),
        (
            !value.metadata.is_empty(),
            "metadata",
            "OTIO track metadata is excluded from the closed canonical IR",
        ),
        (
            !value.name.is_empty(),
            "name",
            "VEAC tracks have stable IDs but no display-name field",
        ),
    ] {
        if present {
            report.push(pointer, field, reason, false);
        }
    }
}

pub(super) fn gap(value: &OtioItem, pointer: &str, report: &mut OtioLossReport) {
    let OtioItem::Gap {
        name,
        metadata,
        effects,
        markers,
        enabled,
        extra,
        ..
    } = value
    else {
        return;
    };
    for (present, field, reason) in [
        (
            !name.is_empty(),
            "name",
            "VEAC gaps are implicit timeline space",
        ),
        (
            !metadata.is_empty(),
            "metadata",
            "implicit VEAC gaps carry no metadata",
        ),
        (
            !effects.is_empty(),
            "effects",
            "implicit VEAC gaps carry no effects",
        ),
        (
            !markers.is_empty(),
            "markers",
            "implicit VEAC gaps carry no markers",
        ),
        (
            !*enabled,
            "enabled",
            "implicit VEAC gaps have no enabled state",
        ),
        (
            !extra.is_empty(),
            "extra",
            "unknown OTIO gap fields are not mapped",
        ),
    ] {
        if present {
            report.push(pointer, field, reason, false);
        }
    }
}

pub(super) fn clip(value: &OtioItem, pointer: &str, report: &mut OtioLossReport) {
    let OtioItem::Clip {
        name,
        media_reference,
        ..
    } = value
    else {
        return;
    };
    if !name.is_empty() {
        report.push(
            pointer,
            "name",
            "explicit bindings choose the canonical item ID",
            false,
        );
    }
    if let OtioMediaReference::External {
        available_range,
        metadata,
        extra,
        ..
    } = media_reference
    {
        for (present, field) in [
            (available_range.is_some(), "media_reference.available_range"),
            (!metadata.is_empty(), "media_reference.metadata"),
            (!extra.is_empty(), "media_reference.extra"),
        ] {
            if present {
                report.push(
                    pointer,
                    field,
                    "OTIO media-reference detail is supplied by the material binding",
                    false,
                );
            }
        }
    }
}
