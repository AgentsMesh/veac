#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
cd "$ROOT"

python3 scripts/stdlib_codegen.py --check

python3 - "$ROOT" <<'PY'
import sys
import copy
import hashlib
import json
import tempfile
from pathlib import Path

root = Path(sys.argv[1])
sys.path.insert(0, str(root / "scripts"))

import stdlib_codegen as codegen
from stdlib_codegen_spec import load_domain_opspec

operations = codegen.load_operations()
signatures = {operation["signature"] for operation in operations}
surfaces = {(operation["receiver"], operation["name"]) for operation in operations}
types = codegen.domain_types(operations)

assert len(operations) == len(signatures) == 582
assert len(surfaces) == 582, "stdlib v6 may not overload callable surfaces"
assert len(types) == len(codegen.declared_types()) == 214

# Markdown remains a signature projection, never an identity source.
markdown_signatures = set()
for family in codegen.FAMILIES:
    path = codegen.DOC_ROOT / f"{family}.md"
    for text in codegen.fenced_signatures(path):
        markdown_signatures.add(codegen.parse_signature(text, family)["signature"])
assert signatures == markdown_signatures
assert len({operation["opcode"] for operation in operations}) == 582
assert len({item["opcode"] for item in types.values()}) == 214
assert types["Context"]["opcode"] == 0x0001
assert next(item for item in operations if item["signature"].startswith("canvas("))["opcode"] == 0x1001
identity = {
    "types": sorted((name, item["opcode"], item["classification"]) for name, item in types.items()),
    "operations": sorted((item["signature"], item["opcode"]) for item in operations),
}
encoded = json.dumps(identity, separators=(",", ":")).encode()
assert hashlib.sha256(encoded).hexdigest() == "0bcc95c17b710d595fb7d6aa8a422ad6b893800e3130b7680bf37cff6b5b2c7f"

audio = codegen.load_family("audio")
assert audio == load_domain_opspec(codegen.OPSPEC_FAMILIES["audio"], "audio")
assert len(audio["types"]) == 12
assert len(audio["operations"]) == 25
assert sum(operation["family"] == "audio" for operation in operations) == 23

stale_documentation = copy.deepcopy(audio)
stale_documentation["types"] = [item["name"] for item in audio["types"]]
stale_documentation["operations"][0]["signature"] = "stale() -> PitchPolicy"
try:
    codegen.check_documentation_projection(
        audio, stale_documentation, codegen.DOC_ROOT / "audio.md"
    )
except ValueError as error:
    assert "signature projection is stale" in str(error)
else:
    raise AssertionError("stale audio Markdown projection was accepted")

rendered = codegen.render_all(operations, types)
assert rendered == codegen.render_all(list(reversed(operations)), dict(reversed(types.items())))
for path, source in rendered.items():
    occurrences = source.count("PrimitiveType")
    assert occurrences != 1, f"unused PrimitiveType import rendered in {path}"

manifest = json.loads(codegen.OPSPEC_FAMILIES["audio"].read_text(encoding="utf-8"))
shard = codegen.OPSPEC_FAMILIES["audio"].parent / manifest["operation_shards"][0]
source = json.loads(shard.read_text(encoding="utf-8"))
def failure(changed_manifest, changed_source, message, family="audio", known=None):
    with tempfile.TemporaryDirectory(prefix="veac-domain-opspec-test-") as temporary:
        directory = Path(temporary)
        path = directory / "family.json"
        path.write_text(json.dumps(changed_manifest), encoding="utf-8")
        (directory / "operations.json").write_text(json.dumps(changed_source), encoding="utf-8")
        try:
            load_domain_opspec(path, family, known)
        except ValueError as error:
            assert message in str(error), str(error)
        else:
            raise AssertionError(f"invalid Domain opspec was accepted: {message}")

invalid = copy.deepcopy(manifest)
invalid["unknown"] = True
failure(invalid, source, "fields must be")
invalid = copy.deepcopy(manifest)
invalid["format_version"] = 2
failure(invalid, source, "format_version")
invalid = copy.deepcopy(manifest)
invalid["types"][1]["opcode"] = invalid["types"][0]["opcode"]
failure(invalid, source, "type opcodes")
invalid = copy.deepcopy(source)
invalid["operations"][1]["opcode"] = invalid["operations"][0]["opcode"]
failure(manifest, invalid, "duplicate Domain opspec operation opcode")
invalid = copy.deepcopy(source)
del invalid["operations"][0]["opcode"]
failure(manifest, invalid, "fields must be")
invalid = copy.deepcopy(source)
invalid["operations"][2]["parameters"][0].pop("axis")
failure(manifest, invalid, "fields must be")
invalid = copy.deepcopy(source)
invalid["operations"][0]["semantics"]["instruction"] = "guessed"
failure(manifest, invalid, "instruction")
invalid = copy.deepcopy(source)
invalid["operations"][0]["semantics"]["effect"] = "graph_emit"
failure(manifest, invalid, "inconsistent")
invalid = copy.deepcopy(source)
invalid["operations"][0]["semantics"]["max_stage"] = "temporal"
failure(manifest, invalid, "inconsistent")
invalid = copy.deepcopy(source)
invalid["operations"][-1]["semantics"]["runtime_action"] = "project_entry"
failure(manifest, invalid, "ProjectEntry shape", known=types)
invalid_manifest = copy.deepcopy(manifest)
invalid_manifest["operation_shards"] = ["../escape.json"]
failure(invalid_manifest, source, "invalid root-confined Domain opspec shard")
invalid = copy.deepcopy(source)
invalid["operations"][0]["result"] = "UnknownDomain"
failure(manifest, invalid, "unknown result type", known=types)
invalid = copy.deepcopy(source)
invalid["operations"][2]["parameters"][0]["type"] = "UnknownDomain"
failure(manifest, invalid, "unknown operand muted type", known=types)
invalid = copy.deepcopy(source)
invalid["operations"][-1]["receiver"]["type"] = "UnknownDomain"
failure(manifest, invalid, "unknown DomainType receiver", known=types)
invalid = copy.deepcopy(source)
invalid["operations"][-1]["receiver"]["axis"] = "leaf"
failure(manifest, invalid, "method receiver must use topology axis", known=types)

values_manifest = json.loads(codegen.OPSPEC_FAMILIES["values-animation"].read_text(encoding="utf-8"))
values_path = codegen.OPSPEC_FAMILIES["values-animation"].parent / values_manifest["operation_shards"][0]
values_source = json.loads(values_path.read_text(encoding="utf-8"))
temporal_index = next(
    index for index, item in enumerate(values_source["operations"])
    if item["semantics"]["temporal_lowering"] is not None
)
invalid = copy.deepcopy(values_source)
invalid["operations"][temporal_index]["result"] = "Rect"
failure(values_manifest, invalid, "temporal lowering shape", "values-animation", types)
invalid = copy.deepcopy(values_source)
invalid["operations"][temporal_index]["semantics"]["temporal_lowering"] = "compose_rect"
failure(values_manifest, invalid, "temporal lowering shape", "values-animation", types)

with tempfile.TemporaryDirectory(prefix="veac-domain-shard-test-") as temporary:
    directory = Path(temporary)
    split_manifest = copy.deepcopy(manifest)
    split_manifest["operation_shards"] = ["one.json", "two.json"]
    one = {"format": source["format"], "format_version": source["format_version"],
           "family": source["family"], "operations": [copy.deepcopy(source["operations"][0])]}
    duplicate = copy.deepcopy(source["operations"][1])
    duplicate["opcode"] = one["operations"][0]["opcode"]
    two = {"format": source["format"], "format_version": source["format_version"],
           "family": source["family"], "operations": [duplicate]}
    (directory / "family.json").write_text(json.dumps(split_manifest), encoding="utf-8")
    (directory / "one.json").write_text(json.dumps(one), encoding="utf-8")
    (directory / "two.json").write_text(json.dumps(two), encoding="utf-8")
    try:
        load_domain_opspec(directory / "family.json", "audio")
    except ValueError as error:
        assert "duplicate Domain opspec operation opcode" in str(error)
    else:
        raise AssertionError("cross-shard operation opcode collision was accepted")

with tempfile.TemporaryDirectory(prefix="veac-domain-reorder-test-") as temporary:
    directory = Path(temporary)
    reversed_manifest = copy.deepcopy(manifest)
    reversed_manifest["types"].reverse()
    reversed_source = copy.deepcopy(source)
    reversed_source["operations"].reverse()
    (directory / "family.json").write_text(json.dumps(reversed_manifest), encoding="utf-8")
    (directory / "operations.json").write_text(json.dumps(reversed_source), encoding="utf-8")
    assert load_domain_opspec(directory / "family.json", "audio") == audio

changed = copy.deepcopy(operations)
changed[0]["semantics"]["effect"] = "graph_emit"
assert codegen.render_all(changed, types) != rendered

PY

python3 scripts/tests/stdlib-codegen-collision-contracts.py "$ROOT"
python3 scripts/tests/stdlib-codegen-action-contracts.py "$ROOT"

printf 'stdlib v6 codegen contracts passed\n'
