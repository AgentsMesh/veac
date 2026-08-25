#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
CHECK="$ROOT/scripts/check-architecture-dependencies.py"
CONTRACT="$ROOT/docs/architecture-dependencies.json"
TOOLCHAIN=${RUSTUP_TOOLCHAIN:-1.85.0}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

fail() {
  printf 'architecture dependency test failed: %s\n' "$1" >&2
  exit 1
}

cargo "+$TOOLCHAIN" metadata --locked --format-version 1 --no-deps \
  --manifest-path "$ROOT/Cargo.toml" >"$TMP/metadata.json"
python3 "$CHECK" --contract "$CONTRACT" --metadata "$TMP/metadata.json" >/dev/null

mutate_metadata() {
  local mode=$1
  local output=$2
  python3 - "$TMP/metadata.json" "$output" "$mode" <<'PY'
import copy
import json
import sys
from pathlib import Path

source, output, mode = sys.argv[1:]
document = json.loads(Path(source).read_text())
packages = {package["name"]: package for package in document["packages"]}
if mode == "unclassified":
    package = copy.deepcopy(packages["veac-ir"])
    package["name"] = "veac-unclassified"
    package["id"] = "path+file:///fixture#veac-unclassified@0.1.0"
    package["manifest_path"] = "/fixture/veac-unclassified/Cargo.toml"
    package["dependencies"] = []
    document["packages"].append(package)
    document["workspace_members"].append(package["id"])
elif mode in {"forbidden", "dev-only", "non-member", "build-non-member",
              "dev-non-member"}:
    dependency = copy.deepcopy(packages["veac-lang"]["dependencies"][0])
    external = mode.endswith("non-member")
    dependency["name"] = "veac-external" if external else "veac-lang"
    dependency["path"] = ("/fixture/veac-external" if external else
                          str(Path(packages["veac-lang"]["manifest_path"]).parent))
    dependency["kind"] = ({"build-non-member": "build", "dev-only": "dev",
                           "dev-non-member": "dev"}.get(mode))
    packages["veac-ir"]["dependencies"].append(dependency)
else:
    raise AssertionError(mode)
Path(output).write_text(json.dumps(document))
PY
}

expect_metadata_failure() {
  local mode=$1
  local expected=$2
  local fixture="$TMP/$mode.json"
  local log="$TMP/$mode.log"
  mutate_metadata "$mode" "$fixture"
  if python3 "$CHECK" --contract "$CONTRACT" --metadata "$fixture" >"$log" 2>&1; then
    fail "$mode fixture unexpectedly passed"
  fi
  rg -Fq "$expected" "$log" || fail "$mode fixture did not report: $expected"
}

expect_metadata_failure unclassified 'unclassified workspace packages: veac-unclassified'
expect_metadata_failure forbidden 'forbidden normal dependency: veac-ir (canonical-model) -> veac-lang (language-facade)'
rg -Fq 'workspace dependency cycle: veac-ir -> veac-lang -> veac-ir' "$TMP/forbidden.log" ||
  fail 'workspace cycle fixture did not report its cycle'
expect_metadata_failure non-member \
  'non-member local normal dependency: veac-ir -> /fixture/veac-external'
expect_metadata_failure build-non-member \
  'non-member local build dependency: veac-ir -> /fixture/veac-external'

mutate_metadata dev-only "$TMP/dev-only.json"
python3 "$CHECK" --contract "$CONTRACT" --metadata "$TMP/dev-only.json" >/dev/null ||
  fail 'dev dependencies must remain outside the production architecture graph'
mutate_metadata dev-non-member "$TMP/dev-non-member.json"
python3 "$CHECK" --contract "$CONTRACT" --metadata "$TMP/dev-non-member.json" >/dev/null ||
  fail 'non-member dev dependencies must remain outside the production architecture graph'

python3 - "$CONTRACT" "$TMP/cycle-contract.json" <<'PY'
import json
import sys
from pathlib import Path

source, output = map(Path, sys.argv[1:])
document = json.loads(source.read_text())
layers = {layer["name"]: layer for layer in document["layers"]}
layers["canonical-model"]["may_depend_on"].append("language-facade")
output.write_text(json.dumps(document))
PY
if python3 "$CHECK" --contract "$TMP/cycle-contract.json" \
    --metadata "$TMP/metadata.json" >"$TMP/contract-cycle.log" 2>&1; then
  fail 'contract cycle fixture unexpectedly passed'
fi
rg -Fq 'contract layer cycle:' "$TMP/contract-cycle.log" ||
  fail 'contract cycle fixture did not report its cycle'

printf 'architecture dependency contracts passed\n'
