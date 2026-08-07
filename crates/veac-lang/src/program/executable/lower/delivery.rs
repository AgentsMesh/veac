use crate::program::expression::runtime::domain_graph::{FrozenDomainGraph, FrozenEntity};
use crate::program::expression::Value;
use crate::program::{DomainOperationId as Op, DomainType};
use veac_ir::{Deliverable, DeliverableKind, DeliverableTarget, RasterSettings, RenderConfig};

use super::error::ExecutableLowerError;
use super::{id, value};

mod auxiliary;
mod common;
mod hls;
mod video;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    entity: FrozenEntity<'_>,
    timebase: u32,
) -> Result<RenderConfig, ExecutableLowerError> {
    let values = value::entity_operands(entity, DomainType::Delivery, Op::Delivery)?;
    let [_, sequence, raster, artifacts] = values else {
        return Err(malformed());
    };
    let path = entity.logical_path().ok_or_else(malformed)?;
    let mut deliverables = value::list(Some(artifacts))?
        .iter()
        .map(|source| deliverable(graph, source, timebase, &path))
        .collect::<Result<Vec<_>, _>>()?;
    deliverables.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    Ok(RenderConfig {
        id: id::delivery(&path),
        sequence_id: common::sequence_ref(graph, sequence)?,
        raster: raster_settings(graph, raster)?,
        deliverables,
    })
}

fn deliverable(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    delivery_path: &[&str],
) -> Result<Deliverable, ExecutableLowerError> {
    let (operation, values) = value::description(graph, source)?;
    let key = value::identifier(values.first())?;
    let mut path = delivery_path.to_vec();
    path.push(key);
    let target = target(graph, values.get(1).ok_or_else(malformed)?)?;
    let kind = match operation {
        Op::DeliverableVideo => DeliverableKind::Video(video::delivery(graph, values)?),
        Op::DeliverableImageFrames => auxiliary::image_frames(graph, values)?,
        Op::DeliverableCaptionSidecar => auxiliary::caption(graph, values)?,
        Op::DeliverableAudioStem => auxiliary::stem(graph, values)?,
        Op::DeliverableMp3 => auxiliary::mp3(graph, values)?,
        Op::DeliverableScope => auxiliary::scope(graph, values, timebase)?,
        Op::DeliverableGif => auxiliary::gif(graph, values)?,
        Op::DeliverableStill => auxiliary::still(graph, values, timebase)?,
        Op::DeliverableHls => hls::package(graph, values, timebase, &path)?,
        _ => return Err(malformed()),
    };
    Ok(Deliverable {
        id: id::deliverable(&path),
        target,
        kind,
    })
}

fn raster_settings(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<RasterSettings>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::RasterNone, []) => Ok(None),
        (Op::RasterSettings, [canvas, rate, captions]) => Ok(Some(RasterSettings {
            width: common::canvas(graph, canvas)?.0,
            height: common::canvas(graph, canvas)?.1,
            frame_rate: common::frame_rate(graph, rate)?,
            captions: common::caption_output(graph, captions)?,
        })),
        _ => Err(malformed()),
    }
}

fn target(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<DeliverableTarget, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::DeliveryFile, [name]) => Ok(DeliverableTarget::File {
            name: value::text(Some(name))?.to_owned(),
        }),
        (Op::DeliveryImageSequence, [pattern]) => Ok(DeliverableTarget::ImageSequence {
            pattern: value::text(Some(pattern))?.to_owned(),
        }),
        (Op::DeliveryPackage, [name]) => Ok(DeliverableTarget::Package {
            name: value::text(Some(name))?.to_owned(),
        }),
        _ => Err(malformed()),
    }
}

pub(super) fn malformed() -> ExecutableLowerError {
    value::graph("an executable delivery has invalid typed topology")
}
