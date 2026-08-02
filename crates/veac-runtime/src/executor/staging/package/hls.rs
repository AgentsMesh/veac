use std::collections::BTreeSet;
use std::path::{Component, Path};
use std::time::Instant;

use m3u8_rs::{MediaPlaylist, MediaPlaylistType, Playlist};
use veac_artifact::{
    DeliveryPackageInventory, DeliveryPackageNodeType, MAX_ARTIFACT_METADATA_BYTES,
};

use super::super::directory::Directory;
use crate::executor::deadline;
use crate::RuntimeError;

pub(super) fn validate(
    root: &Directory,
    inventory: &DeliveryPackageInventory,
    limit: Instant,
) -> Result<(), RuntimeError> {
    deadline::ensure(limit)?;
    let master = parse(root, &inventory.entrypoint, limit)?;
    let Playlist::MasterPlaylist(master) = master else {
        return invalid("HLS entrypoint must be a master playlist");
    };
    if master.variants.is_empty()
        || !master.alternatives.is_empty()
        || !master.session_key.is_empty()
    {
        return invalid("HLS master playlist has an unsupported or empty stream graph");
    }
    let mut reachable = BTreeSet::from([inventory.entrypoint.clone()]);
    let mut media_paths = BTreeSet::new();
    for variant in master.variants {
        let path = resolve(&inventory.entrypoint, &variant.uri)?;
        if !media_paths.insert(path.clone()) {
            return invalid("HLS master playlist contains a duplicate media playlist");
        }
        reachable.insert(path.clone());
        let playlist = parse(root, &path, limit)?;
        let Playlist::MediaPlaylist(media) = playlist else {
            return invalid("HLS master references a non-media playlist");
        };
        validate_media(&path, &media, &mut reachable)?;
    }
    validate_closure(inventory, &reachable)
}

fn validate_media(
    path: &str,
    media: &MediaPlaylist,
    reachable: &mut BTreeSet<String>,
) -> Result<(), RuntimeError> {
    if media.playlist_type != Some(MediaPlaylistType::Vod)
        || !media.end_list
        || media.segments.is_empty()
    {
        return invalid("HLS media playlist must be non-empty VOD with ENDLIST");
    }
    for segment in &media.segments {
        if segment.byte_range.is_some() || segment.key.is_some() || segment.map.is_some() {
            return invalid("HLS media playlist uses an unsupported external segment resource");
        }
        reachable.insert(resolve(path, &segment.uri)?);
    }
    Ok(())
}

fn validate_closure(
    inventory: &DeliveryPackageInventory,
    reachable: &BTreeSet<String>,
) -> Result<(), RuntimeError> {
    for member in &inventory.members {
        let used = match member.node_type {
            DeliveryPackageNodeType::RegularFile => reachable.contains(&member.path),
            DeliveryPackageNodeType::Directory => {
                let prefix = format!("{}/", member.path);
                reachable.iter().any(|path| path.starts_with(&prefix))
            }
        };
        if !used {
            return invalid("HLS package contains an unreachable extra member");
        }
    }
    if reachable.iter().all(|path| {
        inventory.members.iter().any(|member| {
            member.path == *path && member.node_type == DeliveryPackageNodeType::RegularFile
        })
    }) {
        Ok(())
    } else {
        invalid("HLS playlist references a missing package member")
    }
}

fn parse(root: &Directory, path: &str, limit: Instant) -> Result<Playlist, RuntimeError> {
    deadline::ensure(limit)?;
    let bytes = read_relative(root, path, MAX_ARTIFACT_METADATA_BYTES)?;
    m3u8_rs::parse_playlist_res(&bytes)
        .map_err(|_| RuntimeError::new(format!("invalid HLS playlist {path}")))
}

fn read_relative(root: &Directory, path: &str, maximum: u64) -> Result<Vec<u8>, RuntimeError> {
    let components = Path::new(path)
        .components()
        .map(|value| match value {
            Component::Normal(value) => value
                .to_str()
                .ok_or_else(|| RuntimeError::new("HLS path must be valid UTF-8")),
            _ => Err(RuntimeError::new("HLS path must be safe and relative")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    read_components(root, &components, maximum)
}

fn read_components(
    directory: &Directory,
    components: &[&str],
    maximum: u64,
) -> Result<Vec<u8>, RuntimeError> {
    match components {
        [name] => directory.read_bounded(name, maximum).map(|value| value.0),
        [first, rest @ ..] => read_components(&directory.child(first)?, rest, maximum),
        [] => invalid("HLS path may not be empty"),
    }
}

fn resolve(base: &str, uri: &str) -> Result<String, RuntimeError> {
    if uri.is_empty()
        || uri.contains(['\\', '?', '#', '%'])
        || uri.contains("://")
        || uri.contains(':')
        || uri.starts_with("//")
        || uri.chars().any(char::is_control)
    {
        return invalid("HLS URI must be a local relative path");
    }
    let uri = Path::new(uri);
    if !uri
        .components()
        .all(|value| matches!(value, Component::Normal(_)))
    {
        return invalid("HLS URI must not escape its package");
    }
    let path = Path::new(base).parent().unwrap_or(Path::new("")).join(uri);
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| RuntimeError::new("HLS URI must be valid UTF-8"))
}

fn invalid<T>(message: &str) -> Result<T, RuntimeError> {
    Err(RuntimeError::new(message))
}

#[cfg(test)]
#[path = "hls/tests.rs"]
mod tests;
