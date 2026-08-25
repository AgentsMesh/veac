#!/usr/bin/env python3
"""Cross-family identity collision contracts for the Domain opspec loader."""

import copy
import sys
from pathlib import Path

root = Path(sys.argv[1])
sys.path.insert(0, str(root / "scripts"))
import stdlib_codegen as codegen

original = codegen.load_family
first = original(codegen.FAMILIES[0])
second = original(codegen.FAMILIES[1])

operation_collision = copy.deepcopy(second)
operation_collision["operations"][0]["opcode"] = first["operations"][0]["opcode"]
codegen.load_family = lambda family, known=None: (
    first if family == codegen.FAMILIES[0]
    else operation_collision if family == codegen.FAMILIES[1]
    else original(family)
)
try:
    codegen.load_operations()
except ValueError as error:
    assert "duplicate Domain operation opcode" in str(error)
else:
    raise AssertionError("cross-family operation opcode collision was accepted")

type_collision = copy.deepcopy(second)
type_collision["types"][0]["opcode"] = first["types"][0]["opcode"]
codegen.load_family = lambda family, known=None: (
    first if family == codegen.FAMILIES[0]
    else type_collision if family == codegen.FAMILIES[1]
    else original(family)
)
try:
    codegen.domain_types()
except ValueError as error:
    assert "duplicate DomainType opcode" in str(error)
else:
    raise AssertionError("cross-family DomainType opcode collision was accepted")
