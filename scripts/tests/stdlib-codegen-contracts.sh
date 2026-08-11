#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
cd "$ROOT"

python3 scripts/stdlib_codegen.py --check

python3 - "$ROOT" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1])
sys.path.insert(0, str(root / "scripts"))

import stdlib_codegen as codegen

operations = codegen.load_operations()
signatures = {operation["signature"] for operation in operations}
surfaces = {(operation["receiver"], operation["name"]) for operation in operations}
types = codegen.domain_types(operations)

assert len(operations) == len(signatures) == 582
assert len(surfaces) == 582, "stdlib v6 may not overload callable surfaces"
assert len(types) == len(codegen.declared_types()) == 214

rendered = codegen.render_all(operations, types, codegen.action, codegen.axis)
for path, source in rendered.items():
    occurrences = source.count("PrimitiveType")
    assert occurrences != 1, f"unused PrimitiveType import rendered in {path}"
PY

printf 'stdlib v6 codegen contracts passed\n'
