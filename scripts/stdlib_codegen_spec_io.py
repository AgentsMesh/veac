"""Closed JSON and confined shard loading for Domain opspecs."""

from __future__ import annotations

import json
from pathlib import Path, PurePosixPath


def read_json(path: Path):
    def closed_object(pairs):
        value = {}
        for key, item in pairs:
            if key in value:
                raise ValueError(f"duplicate JSON key `{key}` in {path}")
            value[key] = item
        return value

    if path.is_symlink() or not path.is_file():
        raise ValueError(f"Domain opspec must be a regular non-symlink file: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=closed_object)
    except json.JSONDecodeError as error:
        raise ValueError(f"invalid Domain opspec JSON in {path}: {error}") from error


def shard_path(manifest: Path, value: str) -> Path:
    path = PurePosixPath(value) if isinstance(value, str) else PurePosixPath()
    valid = (
        isinstance(value, str)
        and value.endswith(".json")
        and len(value) <= 255
        and not path.is_absolute()
        and len(path.parts) == 1
        and path.parts[0] not in {"", ".", "..", "family.json"}
        and not any(character in value for character in "\\:\0")
        and not any(ord(character) < 0x20 or ord(character) == 0x7f for character in value)
    )
    if not valid:
        raise ValueError(f"invalid root-confined Domain opspec shard: {value!r}")
    result = manifest.parent / value
    if result.is_symlink() or not result.is_file():
        raise ValueError(f"Domain opspec shard is missing or a symlink: {result}")
    return result
