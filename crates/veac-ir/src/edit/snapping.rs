use std::collections::BTreeSet;

use crate::*;

mod types;

pub use types::*;

use super::diagnostic;

pub fn snap_candidates(
    project: &ProjectEnvelope,
    request: &SnapRequest,
) -> Result<Vec<SnapCandidate>, Diagnostic> {
    if let Err(errors) = validate(project) {
        let error = errors
            .into_diagnostics()
            .into_iter()
            .next()
            .unwrap_or_else(|| {
                diagnostic(
                    "INVALID_PROJECT",
                    project.project.id.as_str(),
                    "/project",
                    "project validation failed",
                )
            });
        return Err(error);
    }
    valid_request(project, request)?;
    let sequence = project
        .project
        .sequences
        .iter()
        .find(|value| value.id == request.sequence_id)
        .ok_or_else(|| snap_error(&request.sequence_id, "sequence does not exist"))?;
    let excluded: BTreeSet<_> = request.excluded_items.iter().collect();
    let mut result = Vec::new();
    if request.include_clip_edges {
        for track in &sequence.tracks {
            for clip in &track.clips {
                if excluded.contains(&clip.id) {
                    continue;
                }
                push_if_close(
                    &mut result,
                    request,
                    clip.record_range.start,
                    SnapTarget::ClipStart {
                        track_id: track.id.clone(),
                        item_id: clip.id.clone(),
                    },
                )?;
                let end = clip
                    .record_range
                    .end()
                    .map_err(|_| snap_error(&clip.id, "clip range is invalid"))?;
                push_if_close(
                    &mut result,
                    request,
                    end,
                    SnapTarget::ClipEnd {
                        track_id: track.id.clone(),
                        item_id: clip.id.clone(),
                    },
                )?;
            }
        }
    }
    if request.include_frame_grid {
        frame_candidates(project, sequence, request, &mut result)?;
    }
    result.sort_by(|left, right| {
        left.distance
            .value
            .cmp(&right.distance.value)
            .then_with(|| left.time.value.cmp(&right.time.value))
            .then_with(|| left.target.cmp(&right.target))
    });
    Ok(result)
}

fn valid_request(project: &ProjectEnvelope, request: &SnapRequest) -> Result<(), Diagnostic> {
    let timebase = project.project.timebase;
    let valid = request.time.is_valid()
        && request.tolerance.is_valid()
        && request.time.timescale == timebase
        && request.tolerance.timescale == timebase
        && request.tolerance.value >= 0;
    if valid {
        Ok(())
    } else {
        Err(snap_error(
            &request.sequence_id,
            "snap time and nonnegative tolerance must use the project timebase",
        ))
    }
}

fn frame_candidates(
    project: &ProjectEnvelope,
    sequence: &Sequence,
    request: &SnapRequest,
    result: &mut Vec<SnapCandidate>,
) -> Result<(), Diagnostic> {
    let numerator = i128::from(sequence.settings.frame_rate.numerator);
    let scaled =
        i128::from(project.project.timebase) * i128::from(sequence.settings.frame_rate.denominator);
    if numerator <= 0 || scaled % numerator != 0 {
        return Err(diagnostic(
            "INEXACT_FRAME_GRID",
            sequence.id.as_str(),
            "/snap/include_frame_grid",
            "project timebase cannot exactly represent this sequence frame grid",
        ));
    }
    let step = i64::try_from(scaled / numerator)
        .map_err(|_| snap_error(&sequence.id, "frame grid is outside the safe time range"))?;
    let floor = request.time.value.div_euclid(step).max(0);
    let ceil = if request.time.value.rem_euclid(step) == 0 {
        floor
    } else {
        floor.saturating_add(1)
    };
    for index in [floor, ceil].into_iter().collect::<BTreeSet<_>>() {
        let value = index
            .checked_mul(step)
            .ok_or_else(|| snap_error(&sequence.id, "frame grid time overflowed"))?;
        let time = RationalTime::new(value, project.project.timebase)
            .map_err(|_| snap_error(&sequence.id, "frame grid time is unsafe"))?;
        push_if_close(
            result,
            request,
            time,
            SnapTarget::FrameGrid { frame_index: index },
        )?;
    }
    Ok(())
}

fn push_if_close(
    result: &mut Vec<SnapCandidate>,
    request: &SnapRequest,
    time: RationalTime,
    target: SnapTarget,
) -> Result<(), Diagnostic> {
    let delta = i128::from(time.value) - i128::from(request.time.value);
    let distance_value = i64::try_from(delta.abs())
        .map_err(|_| snap_error(&request.sequence_id, "snap distance overflowed"))?;
    if distance_value <= request.tolerance.value {
        result.push(SnapCandidate {
            time,
            distance: RationalTime::new(distance_value, request.time.timescale)
                .map_err(|_| snap_error(&request.sequence_id, "snap distance is unsafe"))?,
            target,
        });
    }
    Ok(())
}

fn snap_error(id: &impl ToString, message: &str) -> Diagnostic {
    diagnostic("INVALID_SNAP_REQUEST", &id.to_string(), "/snap", message)
}
