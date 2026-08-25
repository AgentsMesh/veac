"""Strict Domain opspec v3 type and operation value parsing."""

from stdlib_codegen_schema import (
    AXES,
    CLASSIFICATIONS,
    EFFECTS,
    INSTRUCTIONS,
    RUNTIME_ACTIONS,
    STAGES,
    TEMPORAL_LOWERINGS,
    closed_enum,
    opcode,
)
from stdlib_codegen_spec import CALLABLE, DOMAIN_TYPE, VALUE_TYPE, _matched, _object, _unique

TYPE_KEYS = {"name", "opcode", "classification"}
OPERATION_KEYS = {"opcode", "receiver", "name", "parameters", "result", "semantics"}
RECEIVER_KEYS = {"type", "axis"}
PARAMETER_KEYS = {"name", "type", "axis"}
SEMANTICS_KEYS = {
    "instruction", "runtime_action", "effect", "max_stage", "temporal_lowering",
}


def types(value):
    if not isinstance(value, list) or not value:
        raise ValueError("Domain opspec types must be a non-empty array")
    output = []
    for index, item in enumerate(value):
        label = f"types[{index}]"
        _object(item, TYPE_KEYS, label)
        output.append({
            "name": _matched(item["name"], DOMAIN_TYPE, f"{label}.name"),
            "opcode": opcode(item["opcode"], f"{label}.opcode"),
            "classification": closed_enum(
                item["classification"], CLASSIFICATIONS, f"{label}.classification"
            ),
        })
    _unique([item["name"] for item in output], "Domain opspec type names")
    _unique([item["opcode"] for item in output], "Domain opspec type opcodes")
    return output


def operations(value, family, surfaces=None, opcodes=None):
    if not isinstance(value, list) or not value:
        raise ValueError("Domain opspec operations must be a non-empty array")
    output = []
    surfaces = surfaces if surfaces is not None else set()
    opcodes = opcodes if opcodes is not None else set()
    for index, item in enumerate(value):
        label = f"operations[{index}]"
        _object(item, OPERATION_KEYS, label)
        receiver, receiver_axis = _receiver(item["receiver"], label)
        name = _matched(item["name"], CALLABLE, f"{label}.name")
        surface = (receiver, name)
        if surface in surfaces:
            raise ValueError(f"duplicate Domain opspec callable {surface}")
        surfaces.add(surface)
        operation_opcode = opcode(item["opcode"], f"{label}.opcode")
        if operation_opcode in opcodes:
            raise ValueError(f"duplicate Domain opspec operation opcode {operation_opcode}")
        opcodes.add(operation_opcode)
        output.append(operation(
            family,
            operation_opcode,
            receiver,
            name,
            parameters(item["parameters"], label),
            _matched(item["result"], DOMAIN_TYPE, f"{label}.result"),
            _semantics(item["semantics"], label),
            receiver_axis,
        ))
    _unique([item["opcode"] for item in output], "Domain opspec operation opcodes")
    return output


def parameters(value, operation_label):
    if not isinstance(value, list):
        raise ValueError(f"{operation_label}.parameters must be an array")
    output = []
    for index, item in enumerate(value):
        label = f"{operation_label}.parameters[{index}]"
        _object(item, PARAMETER_KEYS, label)
        output.append((
            _matched(item["name"], CALLABLE, f"{label}.name"),
            _matched(item["type"], VALUE_TYPE, f"{label}.type"),
            closed_enum(item["axis"], AXES, f"{label}.axis"),
        ))
    _unique([name for name, _, _ in output], f"{operation_label} parameter names")
    return output


def _receiver(value, operation_label):
    if value is None:
        return None, None
    label = f"{operation_label}.receiver"
    _object(value, RECEIVER_KEYS, label)
    return (
        _matched(value["type"], DOMAIN_TYPE, f"{label}.type"),
        closed_enum(value["axis"], AXES, f"{label}.axis"),
    )


def _semantics(value, operation_label):
    label = f"{operation_label}.semantics"
    _object(value, SEMANTICS_KEYS, label)
    lowering = value["temporal_lowering"]
    if lowering is not None:
        lowering = closed_enum(lowering, TEMPORAL_LOWERINGS, f"{label}.temporal_lowering")
    result = {
        "instruction": closed_enum(value["instruction"], INSTRUCTIONS, f"{label}.instruction"),
        "runtime_action": closed_enum(value["runtime_action"], RUNTIME_ACTIONS, f"{label}.runtime_action"),
        "effect": closed_enum(value["effect"], EFFECTS, f"{label}.effect"),
        "max_stage": closed_enum(value["max_stage"], STAGES, f"{label}.max_stage"),
        "temporal_lowering": lowering,
    }
    pure = (
        result["instruction"], result["runtime_action"], result["effect"]
    ) == ("domain_construct", "description", "pure")
    graph = (
        result["instruction"] == "graph_emit"
        and result["runtime_action"] != "description"
        and result["effect"] == "graph_emit"
    )
    temporal = (result["max_stage"] == "temporal") == (lowering is not None)
    if not (pure or graph) or not temporal:
        raise ValueError(f"inconsistent {label} execution contract")
    return result


def operation(family, operation_opcode, receiver, name, args, result, semantics, receiver_axis=None):
    prefix = f"{receiver}." if receiver else ""
    signature_args = ", ".join(f"{key}: {kind}" for key, kind, *_ in args)
    return {
        "family": family,
        "opcode": operation_opcode,
        "receiver": receiver,
        "receiver_axis": receiver_axis,
        "name": name,
        "args": args,
        "result": result,
        "signature": f"{prefix}{name}({signature_args}) -> {result}",
        "semantics": semantics,
    }
