use crate::authoring::{MulticamDecl, MulticamSwitchDecl, MulticamSyncBasisDecl, TypedReference};
use veac_ir::{
    ClipSource, MulticamAngle, MulticamGroup, MulticamSwitch, MulticamSync, MulticamSyncBasis,
};

use super::{context::Context, ids, value};

pub(super) fn group(ctx: &mut Context, value: &MulticamDecl) -> Option<MulticamGroup> {
    let mut angles = value
        .angles
        .iter()
        .map(|angle| {
            Some(MulticamAngle {
                id: ids::angle(ctx, &angle.id)?,
                material_id: ids::material(ctx, &angle.resource.id)?,
                source_offset: value::time(ctx, &angle.source_offset)?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    angles.sort_by(|left, right| left.id.cmp(&right.id));
    Some(MulticamGroup {
        id: ids::multicam(ctx, &value.id)?,
        sync: MulticamSync {
            basis: sync_basis(value.sync.basis),
            reference_angle_id: angle_reference(ctx, &value.sync.reference)?,
        },
        angles,
    })
}

pub(super) fn source(
    ctx: &mut Context,
    group: &TypedReference,
    switches: &[MulticamSwitchDecl],
) -> Option<ClipSource> {
    let switches = switches
        .iter()
        .map(|value| {
            Some(MulticamSwitch {
                angle_id: angle_reference(ctx, &value.angle)?,
                range: value::range(ctx, &value.at, &value.duration)?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ClipSource::Multicam {
        group_id: ids::multicam(ctx, &group.id)?,
        switches,
    })
}

fn angle_reference(ctx: &mut Context, value: &TypedReference) -> Option<veac_ir::MulticamAngleId> {
    ids::angle(ctx, &value.id)
}

fn sync_basis(value: MulticamSyncBasisDecl) -> MulticamSyncBasis {
    match value {
        MulticamSyncBasisDecl::Timecode => MulticamSyncBasis::Timecode,
        MulticamSyncBasisDecl::Audio => MulticamSyncBasis::Audio,
        MulticamSyncBasisDecl::Manual => MulticamSyncBasis::Manual,
    }
}
