#!/usr/bin/env python3
"""Keep physical crate ownership and compiler-query claims in sync."""

import argparse
import json
import re
import sys
from pathlib import Path


DEFAULT_ROOT = Path(__file__).resolve().parents[1]
OWNERSHIP_ROW = re.compile(r"^\| `([^`]+)` \|")


class ContractError(Exception):
    """A repository architecture contract is inconsistent."""


def read(root: Path, relative: str) -> str:
    path = root / relative
    try:
        return path.read_text(encoding="utf-8")
    except OSError as error:
        raise ContractError(f"cannot read {relative}: {error}") from error


def ownership_rows(document: str) -> set[str]:
    section = document.split("## Crate Ownership", 1)
    if len(section) != 2:
        raise ContractError("architecture.md is missing the Crate Ownership section")
    section = section[1].split("## ", 1)[0]
    return {match.group(1) for line in section.splitlines()
            if (match := OWNERSHIP_ROW.match(line))}


def contract_packages(root: Path) -> tuple[set[str], set[str]]:
    try:
        document = json.loads(read(root, "docs/architecture-dependencies.json"))
    except json.JSONDecodeError as error:
        raise ContractError(f"invalid architecture dependency JSON: {error}") from error
    active = set()
    planned = set()
    for layer in document.get("layers", []):
        active.update(layer.get("packages", []))
        planned.update(layer.get("planned_packages", []))
    if active & planned:
        names = ", ".join(sorted(active & planned))
        raise ContractError(f"crate is both active and planned: {names}")
    return active, planned


def check(root: Path) -> None:
    architecture = read(root, "docs/architecture.md")
    query_rfc = read(root, "docs/rfcs/compiler-query-database.md")
    boundaries = read(root, "docs/language-design/compiler-boundaries.md")
    makefile = read(root, "Makefile")
    structure = read(root, "scripts/check-rust-structure.sh")
    active, planned = contract_packages(root)
    rows = ownership_rows(architecture)
    missing = active - rows
    if missing:
        raise ContractError(
            "active crates missing from architecture ownership table: "
            + ", ".join(sorted(missing))
        )
    documented_planned = planned & rows
    if documented_planned:
        raise ContractError(
            "planned crates are presented as physically owned: "
            + ", ".join(sorted(documented_planned))
        )
    if "facade over language model/domain spec" not in architecture:
        raise ContractError("architecture.md must identify veac-lang as a facade")
    for package in sorted(planned):
        if f"`{package}`" not in architecture:
            raise ContractError(f"planned crate is undocumented: {package}")

    required_query_markers = (
        "当前实现提供的是有界、可丢弃的性能缓存",
        "真正按 module/declaration 复用 interface、HIR/Core",
        "属于下一阶段设计",
    )
    for marker in required_query_markers:
        if marker not in query_rfc:
            raise ContractError(f"query RFC is missing boundary marker: {marker}")
    if "当前 interface 不是独立 module query" not in boundaries:
        raise ContractError("compiler-boundaries.md must state the current query scope")
    if "Compiler Query Database" not in architecture or "semantic oracle" not in architecture:
        raise ContractError("architecture.md must link the query RFC and clean-build oracle")
    if "bash scripts/check-rust-structure.sh" not in makefile:
        raise ContractError("Makefile structure target is not wired to Rust structure checks")
    if "check-architecture-boundaries.py" not in structure:
        raise ContractError("Rust structure gate does not run architecture boundary checks")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    arguments = parser.parse_args()
    try:
        check(arguments.root.resolve())
    except ContractError as error:
        print(f"architecture boundary contract failed: {error}", file=sys.stderr)
        return 1
    print("architecture boundary contract passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
