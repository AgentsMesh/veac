#!/usr/bin/env python3
"""Generate executable standard-library v6 Rust tables from family sources."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

from stdlib_codegen_io import write_or_check
from stdlib_codegen_render import render_all
from stdlib_codegen_spec import load_domain_opspec, parse_signature
from stdlib_codegen_validation import validate_operations

ROOT = Path(__file__).resolve().parents[1]
DOC_ROOT = ROOT / "docs/language-design/executable-stdlib-v6"
SPEC_ROOT = ROOT / "spec/domain"
FAMILIES = (
    "values-animation",
    "settings-materials",
    "sources-generators",
    "visual-masks",
    "color-effects",
    "text",
    "audio",
    "relations-apply",
    "multicam-template-annotation",
    "delivery-video",
    "delivery-auxiliary",
)
OPSPEC_FAMILIES = {family: SPEC_ROOT / family / "family.json" for family in FAMILIES}
def fenced_signatures(path: Path):
    inside = False
    pending: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        if raw == "```veac":
            inside = True
            continue
        if raw == "```" and inside:
            if pending:
                raise ValueError(f"unterminated signature in {path}: {' '.join(pending)}")
            inside = False
            continue
        if not inside or not raw.strip():
            continue
        pending.append(raw.strip())
        joined = " ".join(pending)
        if re.search(r"-> [A-Z][A-Za-z0-9]*$", joined):
            yield re.sub(r"\s+", " ", joined)
            pending.clear()


def documented_types(path: Path):
    text = path.read_text(encoding="utf-8")
    block = re.search(r"## DomainTypes\s+```text\s+(.*?)```", text, re.S)
    if not block:
        raise ValueError(f"{path.name} has no DomainTypes block")
    return re.findall(r"\b[A-Z][A-Za-z0-9]*\b", block.group(1))


def load_family(family: str, known_types=None):
    path = DOC_ROOT / f"{family}.md"
    documentation = {
        "family": family,
        "types": documented_types(path),
        "operations": [parse_signature(text, family) for text in fenced_signatures(path)],
    }
    specification = load_domain_opspec(OPSPEC_FAMILIES[family], family, known_types)
    check_documentation_projection(specification, documentation, path)
    return specification


def check_documentation_projection(specification, documentation, path: Path):
    expected_types = {item["name"] for item in specification["types"]}
    actual_types = set(documentation["types"])
    if expected_types != actual_types or len(actual_types) != len(documentation["types"]):
        raise ValueError(f"DomainTypes projection is stale in {path}")
    expected = {item["signature"] for item in specification["operations"]}
    actual = {item["signature"] for item in documentation["operations"]}
    if expected != actual or len(actual) != len(documentation["operations"]):
        raise ValueError(f"signature projection is stale in {path}")


def load_operations():
    types = domain_types()
    operations = []
    surfaces = {}
    signatures = {}
    opcodes = {}
    for family in FAMILIES:
        for operation in load_family(family, types)["operations"]:
            text = operation["signature"]
            surface = (operation["receiver"], operation["name"])
            previous = surfaces.get(surface)
            if previous and previous != text:
                raise ValueError(f"overloaded Surface callable {surface}: {previous} / {text}")
            surfaces[surface] = text
            previous_opcode = opcodes.get(operation["opcode"])
            if previous_opcode and previous_opcode != text:
                raise ValueError(
                    f"duplicate Domain operation opcode {operation['opcode']}: "
                    f"{previous_opcode} / {text}"
                )
            opcodes[operation["opcode"]] = text
            previous_operation = signatures.get(text)
            if previous_operation:
                if _identity(previous_operation) != _identity(operation):
                    raise ValueError(f"conflicting repeated Domain operation: {text}")
            else:
                signatures[text] = operation
                operations.append(operation)
    if len(operations) != 582:
        raise ValueError(f"expected 582 unique signatures, found {len(operations)}")
    operations = sorted(operations, key=lambda operation: operation["opcode"])
    validate_operations(operations, types)
    return operations


def _identity(operation):
    return {key: value for key, value in operation.items() if key != "family"}


def domain_types(_operations=None):
    values = {}
    opcodes = {}
    for family in FAMILIES:
        for item in load_family(family)["types"]:
            name = item["name"]
            if name in values:
                raise ValueError(f"duplicate declared DomainType: {name}")
            previous = opcodes.get(item["opcode"])
            if previous:
                raise ValueError(
                    f"duplicate DomainType opcode {item['opcode']}: {previous} / {name}"
                )
            opcodes[item["opcode"]] = name
            values[name] = {**item, "family": family}
    if len(values) != 214:
        raise ValueError(f"expected 214 declared DomainTypes, found {len(values)}")
    return dict(sorted(values.items(), key=lambda item: item[1]["opcode"]))


def declared_types():
    values = set()
    for family in FAMILIES:
        values.update(item["name"] for item in load_family(family)["types"])
    return values


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    operations = load_operations()
    rendered = render_all(operations, domain_types(operations))
    return write_or_check(rendered, args.check)


if __name__ == "__main__":
    raise SystemExit(main())
