#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CONTRACT = ROOT / "docs/architecture-dependencies.json"


class ContractError(Exception):
    pass


def load_json(path):
    try:
        with path.open(encoding="utf-8") as source:
            return json.load(source)
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot read {path}: {error}") from error


def cargo_metadata():
    toolchain = os.environ.get("RUSTUP_TOOLCHAIN", "1.85.0")
    command = ["cargo", f"+{toolchain}", "metadata", "--locked",
               "--format-version", "1", "--no-deps",
               "--manifest-path", str(ROOT / "Cargo.toml")]
    try:
        result = subprocess.run(command, check=True, capture_output=True, text=True)
        return json.loads(result.stdout)
    except (OSError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        detail = getattr(error, "stderr", "").strip()
        raise ContractError(f"cargo metadata failed: {detail or error}") from error


def find_cycle(graph):
    state = {}
    stack = []
    positions = {}

    def visit(node):
        state[node] = 1
        positions[node] = len(stack)
        stack.append(node)
        for target in sorted(graph.get(node, ())):
            if state.get(target, 0) == 0:
                cycle = visit(target)
                if cycle:
                    return cycle
            elif state[target] == 1:
                return stack[positions[target]:] + [target]
        stack.pop()
        positions.pop(node)
        state[node] = 2
        return None

    for node in sorted(graph):
        if state.get(node, 0) == 0:
            cycle = visit(node)
            if cycle:
                return cycle
    return None


def parse_contract(document):
    expected = {"schema_version", "dependency_kinds", "unlisted_packages",
                "cycles", "layers"}
    if not isinstance(document, dict) or set(document) != expected:
        raise ContractError("contract root fields do not match schema version 1")
    if document["schema_version"] != 1:
        raise ContractError("unsupported contract schema_version")
    kinds = document["dependency_kinds"]
    if (not isinstance(kinds, list) or not kinds or
            any(kind not in {"normal", "build"} for kind in kinds)):
        raise ContractError("dependency_kinds must contain normal and/or build")
    if len(kinds) != len(set(kinds)):
        raise ContractError("dependency_kinds contains duplicates")
    if document["unlisted_packages"] != "forbidden":
        raise ContractError("unlisted_packages must be forbidden")
    if document["cycles"] != "forbidden":
        raise ContractError("cycles must be forbidden")

    if not isinstance(document["layers"], list) or not document["layers"]:
        raise ContractError("layers must be a non-empty list")
    layer_fields = {"name", "packages", "planned_packages", "may_depend_on"}
    layers = {}
    package_layers = {}
    required = set()
    for layer in document["layers"]:
        if not isinstance(layer, dict) or set(layer) != layer_fields:
            raise ContractError("layer fields do not match schema version 1")
        name = layer["name"]
        if not isinstance(name, str) or not name or name in layers:
            raise ContractError(f"invalid or duplicate layer: {name!r}")
        for field in ("packages", "planned_packages", "may_depend_on"):
            values = layer[field]
            if (not isinstance(values, list) or
                    any(not isinstance(value, str) or not value for value in values) or
                    len(values) != len(set(values))):
                raise ContractError(f"{name}.{field} must contain unique names")
        layers[name] = set(layer["may_depend_on"])
        for package in layer["packages"] + layer["planned_packages"]:
            if package in package_layers:
                raise ContractError(f"package appears in multiple layers: {package}")
            package_layers[package] = name
        required.update(layer["packages"])

    for name, targets in layers.items():
        unknown = targets - layers.keys()
        if unknown:
            raise ContractError(f"{name} names unknown layers: {', '.join(sorted(unknown))}")
        if name in targets:
            raise ContractError(f"layer may not depend on itself: {name}")
    cycle = find_cycle(layers)
    if cycle:
        raise ContractError(f"contract layer cycle: {' -> '.join(cycle)}")
    return set(kinds), layers, package_layers, required


def workspace_graph(metadata, kinds):
    members = set(metadata.get("workspace_members", []))
    packages = [package for package in metadata.get("packages", [])
                if package.get("id") in members]
    manifests = {}
    names = {}
    for package in packages:
        name = package.get("name")
        manifest = package.get("manifest_path")
        if not name or not manifest or name in names:
            raise ContractError(f"invalid or duplicate workspace package: {name!r}")
        names[name] = package
        manifests[str(Path(manifest).resolve().parent)] = name

    graph = {name: set() for name in names}
    edges = []
    invalid_paths = []
    for name, package in names.items():
        for dependency in package.get("dependencies", []):
            kind = dependency.get("kind") or "normal"
            path = dependency.get("path")
            if kind not in kinds or not path:
                continue
            resolved_path = str(Path(path).resolve())
            target = manifests.get(resolved_path)
            if target:
                graph[name].add(target)
                edges.append((name, target, kind))
            else:
                invalid_paths.append((name, resolved_path, kind))
    return names, graph, edges, invalid_paths


def check(contract, metadata):
    kinds, layers, package_layers, required = parse_contract(contract)
    packages, graph, edges, invalid_paths = workspace_graph(metadata, kinds)
    errors = []
    for source, path, kind in sorted(invalid_paths):
        errors.append(f"non-member local {kind} dependency: {source} -> {path}")
    missing = required - packages.keys()
    unlisted = packages.keys() - package_layers.keys()
    if missing:
        errors.append(f"required packages missing: {', '.join(sorted(missing))}")
    if unlisted:
        errors.append(f"unclassified workspace packages: {', '.join(sorted(unlisted))}")
    for source, target, kind in sorted(edges):
        source_layer = package_layers.get(source)
        target_layer = package_layers.get(target)
        if source_layer and target_layer and target_layer not in layers[source_layer]:
            errors.append(f"forbidden {kind} dependency: {source} ({source_layer}) -> "
                          f"{target} ({target_layer})")
    cycle = find_cycle(graph)
    if cycle:
        errors.append(f"workspace dependency cycle: {' -> '.join(cycle)}")
    if errors:
        raise ContractError("\n".join(errors))


def main():
    parser = argparse.ArgumentParser(description="Check VEAC workspace dependency layers")
    parser.add_argument("--contract", type=Path, default=DEFAULT_CONTRACT)
    parser.add_argument("--metadata", type=Path)
    arguments = parser.parse_args()
    try:
        metadata = load_json(arguments.metadata) if arguments.metadata else cargo_metadata()
        check(load_json(arguments.contract), metadata)
    except ContractError as error:
        print(f"architecture dependency contract failed: {error}", file=sys.stderr)
        return 1
    print("architecture dependency contract passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
