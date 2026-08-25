"""Strict loader for versioned machine-readable Domain family specifications."""

from __future__ import annotations

import re
from pathlib import Path

from stdlib_codegen_spec_io import read_json, shard_path

FORMAT = "veac.domain-family"
SHARD_FORMAT = "veac.domain-operations"
FORMAT_VERSION = 3
DOMAIN_OPSET_VERSION = 8
CANONICAL_TYPE_COUNT = 214
SPEC_ROOT = Path(__file__).resolve().parents[1] / "spec" / "domain"
TOP_LEVEL_KEYS = {
    "format",
    "format_version",
    "domain_opset_version",
    "family",
    "types",
    "operation_shards",
}
SHARD_KEYS = {"format", "format_version", "family", "operations"}
FAMILY = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
CALLABLE = re.compile(r"[a-z][a-z0-9_]*")
DOMAIN_TYPE = re.compile(r"[A-Z][A-Za-z0-9]*")
VALUE_TYPE = re.compile(r"(?:[A-Za-z][A-Za-z0-9]*|list<[A-Za-z][A-Za-z0-9]*>)")
SIGNATURE = re.compile(
    r"^(?:(?P<receiver>[A-Z][A-Za-z0-9]*)\.)?"
    r"(?P<name>[a-z][a-z0-9_]*)\((?P<args>.*)\) -> "
    r"(?P<result>[A-Z][A-Za-z0-9]*)$"
)
PARAMETER = re.compile(
    r"^(?P<name>[a-z][a-z0-9_]*): "
    r"(?P<type>(?:list<)?[A-Za-z][A-Za-z0-9]*(?:>)?)$"
)


def load_domain_opspec(path: Path, expected_family: str, known_types=None):
    from stdlib_codegen_spec_values import types as load_types
    from stdlib_codegen_validation import validate_operations

    if path.is_dir():
        path = path / "family.json"
    value = read_json(path)
    _object(value, TOP_LEVEL_KEYS, "Domain opspec")
    _literal(value["format"], FORMAT, "format")
    _literal(value["format_version"], FORMAT_VERSION, "format_version")
    _literal(
        value["domain_opset_version"],
        DOMAIN_OPSET_VERSION,
        "domain_opset_version",
    )
    family = _matched(value["family"], FAMILY, "family")
    _literal(family, expected_family, "family")
    types = load_types(value["types"])
    shards = _shards(value["operation_shards"])
    operations = []
    surfaces = set()
    opcodes = set()
    for shard in shards:
        operations.extend(_load_shard(shard_path(path, shard), family, surfaces, opcodes))
    result = {
        "family": family,
        "types": sorted(types, key=lambda item: item["opcode"]),
        "operations": sorted(operations, key=lambda item: item["opcode"]),
    }
    validate_operations(result["operations"], known_types or canonical_types())
    return result


def canonical_types():
    from stdlib_codegen_spec_values import types as load_types

    names = {}
    opcodes = {}
    for path in sorted(SPEC_ROOT.glob("*/family.json")):
        value = read_json(path)
        _object(value, TOP_LEVEL_KEYS, f"Domain opspec {path}")
        for item in load_types(value["types"]):
            if item["name"] in names:
                raise ValueError(f"duplicate canonical DomainType: {item['name']}")
            if item["opcode"] in opcodes:
                raise ValueError(f"duplicate canonical DomainType opcode {item['opcode']}")
            names[item["name"]] = item
            opcodes[item["opcode"]] = item["name"]
    if len(names) != CANONICAL_TYPE_COUNT:
        raise ValueError(f"expected {CANONICAL_TYPE_COUNT} canonical DomainTypes, found {len(names)}")
    return names


def parse_signature(text: str, family: str):
    from stdlib_codegen_spec_values import operation

    match = SIGNATURE.fullmatch(text)
    if not match:
        raise ValueError(f"invalid v6 signature: {text}")
    args = []
    if match["args"]:
        for raw in match["args"].split(", "):
            parameter = PARAMETER.fullmatch(raw)
            if not parameter:
                raise ValueError(f"invalid parameter `{raw}` in {text}")
            args.append((parameter["name"], parameter["type"]))
    return operation(family, None, match["receiver"], match["name"], args, match["result"], None)


def _load_shard(path, family, surfaces, opcodes):
    from stdlib_codegen_spec_values import operations

    value = read_json(path)
    _object(value, SHARD_KEYS, f"Domain opspec shard {path.name}")
    _literal(value["format"], SHARD_FORMAT, "shard format")
    _literal(value["format_version"], FORMAT_VERSION, "shard format_version")
    _literal(value["family"], family, "shard family")
    return operations(value["operations"], family, surfaces, opcodes)


def _shards(value):
    if not isinstance(value, list) or not value:
        raise ValueError("operation_shards must be a non-empty array")
    if any(not isinstance(item, str) for item in value):
        raise ValueError("operation_shards entries must be strings")
    _unique(value, "Domain opspec operation_shards")
    return value


def _object(value, keys, label):
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be an object")
    if set(value) != keys:
        raise ValueError(
            f"{label} fields must be {sorted(keys)}; found {sorted(value)}"
        )


def _matched(value, pattern, label):
    if not isinstance(value, str) or not pattern.fullmatch(value):
        raise ValueError(f"invalid {label}: {value!r}")
    return value


def _literal(value, expected, label):
    if type(value) is not type(expected) or value != expected:
        raise ValueError(f"{label} must be {expected!r}; found {value!r}")


def _unique(values, label):
    if len(values) != len(set(values)):
        raise ValueError(f"{label} must be unique")
