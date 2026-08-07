#!/usr/bin/env python3
"""Generate the executable standard-library v6 Rust tables from the normative docs."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

from stdlib_codegen_io import write_or_check
from stdlib_codegen_policy import GRAPH_TYPES, STATIC_TYPES, action, axis
from stdlib_codegen_render import render_all

ROOT = Path(__file__).resolve().parents[1]
DOC_ROOT = ROOT / "docs/language-design/executable-stdlib-v6"
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
SIGNATURE = re.compile(
    r"^(?:(?P<receiver>[A-Z][A-Za-z0-9]*)\.)?"
    r"(?P<name>[a-z][a-z0-9_]*)\((?P<args>.*)\) -> "
    r"(?P<result>[A-Z][A-Za-z0-9]*)$"
)
PARAMETER = re.compile(
    r"^(?P<name>[a-z][a-z0-9_]*): "
    r"(?P<type>(?:list<)?[A-Za-z][A-Za-z0-9]*(?:>)?)$"
)
PRIMITIVES = {
    "int",
    "scalar",
    "time",
    "length",
    "percent",
    "angle",
    "text",
    "color",
    "bool",
    "identifier",
}
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


def parse_signature(text: str, family: str):
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
    return {
        "family": family,
        "receiver": match["receiver"],
        "name": match["name"],
        "args": args,
        "result": match["result"],
        "signature": text,
    }


def load_operations():
    operations = []
    surfaces = {}
    signatures = set()
    for family in FAMILIES:
        path = DOC_ROOT / f"{family}.md"
        for text in fenced_signatures(path):
            operation = parse_signature(text, family)
            surface = (operation["receiver"], operation["name"])
            previous = surfaces.get(surface)
            if previous and previous != text:
                raise ValueError(f"overloaded Surface callable {surface}: {previous} / {text}")
            surfaces[surface] = text
            if text not in signatures:
                signatures.add(text)
                operations.append(operation)
    if len(operations) != 581:
        raise ValueError(f"expected 581 unique signatures, found {len(operations)}")
    return operations


def inner_type(value: str) -> str:
    return value[5:-1] if value.startswith("list<") else value


def domain_types(operations):
    values = {"Context": {"family": None, "classification": "TopologyValue"}}
    for operation in operations:
        names = [operation["result"]] + [inner_type(item[1]) for item in operation["args"]]
        if operation["receiver"]:
            names.append(operation["receiver"])
        for name in names:
            if name in PRIMITIVES or name in values:
                continue
            classification = "LeafValue" if name not in STATIC_TYPES else "TopologyValue"
            if name in {"Project", "Sequence", "Layer", "Item"}:
                classification = "Container"
            elif name in GRAPH_TYPES:
                classification = "GraphEntity"
            values[name] = {"family": operation["family"], "classification": classification}
    declared = declared_types()
    if len(declared) != 214:
        raise ValueError(f"expected 214 declared DomainTypes, found {len(declared)}")
    if set(values) != declared:
        missing = sorted(declared - set(values))
        unknown = sorted(set(values) - declared)
        raise ValueError(f"DomainTypes/signature mismatch; missing={missing}, unknown={unknown}")
    return values


def declared_types():
    values = set()
    for family in FAMILIES:
        text = (DOC_ROOT / f"{family}.md").read_text(encoding="utf-8")
        block = re.search(r"## DomainTypes\s+```text\s+(.*?)```", text, re.S)
        if not block:
            raise ValueError(f"{family}.md has no DomainTypes block")
        values.update(re.findall(r"\b[A-Z][A-Za-z0-9]*\b", block.group(1)))
    return values


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    operations = load_operations()
    rendered = render_all(operations, domain_types(operations), action, axis)
    return write_or_check(rendered, args.check)


if __name__ == "__main__":
    raise SystemExit(main())
