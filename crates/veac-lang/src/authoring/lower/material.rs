use std::collections::BTreeMap;

use crate::authoring::{
    ResourceDecl, ResourceIdentity, ResourceKind, ResourceLocator, ResourceStreams, StreamSelection,
};
use veac_ir::{
    HashAlgorithm, Material, MaterialKind, MaterialSource, MediaIdentity, StreamChoice,
    StreamIntent,
};

use super::{context::Context, ids};

pub fn lower(ctx: &mut Context, value: &ResourceDecl) -> Option<Material> {
    let (source, identity) = locator(&value.locator);
    Some(Material {
        id: ids::material(ctx, &value.id)?,
        kind: match value.kind {
            ResourceKind::Video => MaterialKind::Video,
            ResourceKind::Audio => MaterialKind::Audio,
            ResourceKind::Image => MaterialKind::Image,
            ResourceKind::Font => MaterialKind::Font,
            ResourceKind::Lut1d => MaterialKind::Lut1d,
            ResourceKind::Lut3d => MaterialKind::Lut3d,
        },
        source,
        identity,
        stream_intent: value
            .streams
            .as_ref()
            .map(stream_intent)
            .unwrap_or_else(|| default_stream_intent(value.kind)),
        probe: None,
        metadata: BTreeMap::new(),
    })
}

fn default_stream_intent(kind: ResourceKind) -> StreamIntent {
    match kind {
        ResourceKind::Video => StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Auto,
        },
        ResourceKind::Audio => StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Auto,
        },
        ResourceKind::Image => StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Disabled,
        },
        ResourceKind::Font | ResourceKind::Lut1d | ResourceKind::Lut3d => StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Disabled,
        },
    }
}

fn locator(value: &ResourceLocator) -> (MaterialSource, Option<MediaIdentity>) {
    match value {
        ResourceLocator::Local { path } => (
            MaterialSource::File {
                uri: path.value.clone(),
            },
            None,
        ),
        ResourceLocator::Remote { uri, identity } => (
            MaterialSource::Remote {
                uri: uri.value.clone(),
            },
            Some(match identity {
                ResourceIdentity::Sha256(value) => MediaIdentity {
                    algorithm: HashAlgorithm::Sha256,
                    digest: value.value.clone(),
                },
            }),
        ),
    }
}

fn stream_intent(value: &ResourceStreams) -> StreamIntent {
    StreamIntent {
        video: selection(value.video),
        audio: selection(value.audio),
    }
}

fn selection(value: StreamSelection) -> StreamChoice {
    match value {
        StreamSelection::Auto => StreamChoice::Auto,
        StreamSelection::Disabled => StreamChoice::Disabled,
    }
}
