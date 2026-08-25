#!/usr/bin/env python3
"""Negative contracts for Domain action and Temporal operation shapes."""

import copy
import json
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1])
sys.path.insert(0, str(root / "scripts"))
import stdlib_codegen as codegen
from stdlib_codegen_spec import load_domain_opspec


def fixture(family):
    manifest_path = codegen.OPSPEC_FAMILIES[family]
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    shard_path = manifest_path.parent / manifest["operation_shards"][0]
    return manifest, json.loads(shard_path.read_text(encoding="utf-8"))


def operation(source, name):
    return next(item for item in source["operations"] if item["name"] == name)


def rejects(manifest, source, family, message):
    with tempfile.TemporaryDirectory(prefix="veac-action-contract-") as temporary:
        directory = Path(temporary)
        value = copy.deepcopy(manifest)
        value["operation_shards"] = ["operations.json"]
        (directory / "family.json").write_text(json.dumps(value), encoding="utf-8")
        (directory / "operations.json").write_text(json.dumps(source), encoding="utf-8")
        try:
            load_domain_opspec(directory / "family.json", family)
        except ValueError as error:
            if message not in str(error):
                raise AssertionError(f"expected {message!r}, got {error}") from error
        else:
            raise AssertionError(f"invalid action contract was accepted: {message}")


settings_manifest, settings_source = fixture("settings-materials")
entry = copy.deepcopy(settings_source)
operation(entry, "entry")["parameters"][0]["type"] = "Resource"
rejects(settings_manifest, entry, "settings-materials", "ProjectEntry shape")

attachment = copy.deepcopy(settings_source)
operation(attachment, "with_resource")["semantics"]["runtime_action"] = "project_entry"
rejects(settings_manifest, attachment, "settings-materials", "ProjectEntry shape")

entity = copy.deepcopy(settings_source)
operation(entity, "project")["parameters"][0]["type"] = "ProjectSettings"
rejects(settings_manifest, entity, "settings-materials", "EntityConstructor shape")

relations_manifest, relations_source = fixture("relations-apply")
relation = copy.deepcopy(relations_source)
operation(relation, "relation_transition")["result"] = "Project"
rejects(relations_manifest, relation, "relations-apply", "RelationConstructor shape")

multicam_manifest, multicam_source = fixture("multicam-template-annotation")
update = copy.deepcopy(multicam_source)
operation(update, "with_template")["parameters"][0]["type"] = "Resource"
rejects(multicam_manifest, update, "multicam-template-annotation", "NonOwningUpdate shape")

temporal_manifest, temporal_source = fixture("values-animation")
point = copy.deepcopy(temporal_source)
operation(point, "point")["parameters"][0]["type"] = "scalar"
rejects(temporal_manifest, point, "values-animation", "temporal lowering shape")

unknown = copy.deepcopy(settings_source)
operation(unknown, "project")["result"] = "UnknownType"
rejects(settings_manifest, unknown, "settings-materials", "unknown result type")

print("stdlib Domain action contracts passed")
