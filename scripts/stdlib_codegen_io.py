"""Formatting and deterministic check/write support for generated Rust tables."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUTS = (
    ROOT / "crates/veac-lang-model/src/domain_type/generated",
    ROOT / "crates/veac-domain-spec/src/operation_id/generated",
    ROOT / "crates/veac-domain-spec/src/catalog/generated",
    ROOT / "crates/veac-ir/src/validation/provenance/generated",
)


def formatted(rendered):
    with tempfile.TemporaryDirectory(prefix="veac-stdlib-v6-") as temporary:
        root = Path(temporary)
        staged = []
        for path, text in rendered.items():
            target = root / path.relative_to(ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf-8")
            staged.append(target)
        subprocess.run(
            ["rustfmt", "+1.85.0", "--edition", "2021", *map(str, staged)],
            check=True,
        )
        return {
            ROOT / path.relative_to(root): path.read_text(encoding="utf-8")
            for path in staged
        }


def existing():
    return {
        path: path.read_text(encoding="utf-8")
        for root in OUTPUTS
        for path in root.glob("*.rs")
    }


def write_or_check(rendered, check):
    expected = formatted(rendered)
    oversized = [(path, text.count("\n")) for path, text in expected.items() if text.count("\n") >= 200]
    if oversized:
        raise ValueError(f"formatted generated files exceed 199 lines: {oversized}")
    if check:
        if existing() != expected:
            print("generated executable stdlib v6 tables are stale", file=sys.stderr)
            return 1
        return 0
    for root in OUTPUTS:
        root.mkdir(parents=True, exist_ok=True)
        for path in root.glob("*.rs"):
            path.unlink()
    for path, text in expected.items():
        path.write_text(text, encoding="utf-8")
    return 0
