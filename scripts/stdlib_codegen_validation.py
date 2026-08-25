"""Cross-family validation for explicit Domain opspec contracts."""

PRIMITIVES = {
    "int", "scalar", "time", "length", "percent", "angle", "text", "color",
    "bool", "identifier",
}
GRAPH_CLASSES = {"container", "graph_entity"}
TEMPORAL_SHAPES = {
    "compose_point": ("Point", ("length", "length")),
    "compose_vector": ("Vector", ("scalar", "scalar")),
    "compose_rect": ("Rect", ("scalar", "scalar", "scalar", "scalar")),
}


def validate_operations(operations, types):
    for operation in operations:
        _validate_operation(operation, types)


def _validate_operation(operation, types):
    receiver = operation["receiver"]
    if receiver and receiver not in types:
        raise ValueError(f"unknown DomainType receiver: {receiver}")
    _validate_shape(operation["result"], types, "result")
    if receiver and operation["receiver_axis"] != "topology":
        raise ValueError(f"method receiver must use topology axis: {operation['signature']}")
    for name, value, axis in operation["args"]:
        _validate_shape(value, types, f"operand {name}")
        inner = value[5:-1] if value.startswith("list<") else value
        if inner in types and types[inner]["classification"] != "leaf_value" and axis == "leaf":
            raise ValueError(f"topology DomainType uses leaf axis: {operation['signature']}")
    _validate_semantics(operation, types)


def _validate_shape(value, types, label):
    inner = value[5:-1] if value.startswith("list<") else value
    if inner not in PRIMITIVES and inner not in types:
        raise ValueError(f"unknown {label} type: {value}")


def _validate_semantics(operation, types):
    semantics = operation["semantics"]
    action = semantics["runtime_action"]
    receiver = operation["receiver"]
    result = operation["result"]
    graph = result in types and types[result]["classification"] in GRAPH_CLASSES
    if action == "project_entry":
        expected = [("sequence", "Sequence", "topology")]
        if receiver != "Project" or result != "Project" or operation["args"] != expected:
            raise ValueError(f"ProjectEntry shape is invalid: {operation['signature']}")
    if action == "owned_attachment":
        if not receiver or types[receiver]["classification"] != "container":
            raise ValueError(f"OwnedAttachment receiver shape is invalid: {operation['signature']}")
        args = operation["args"]
        if (result != receiver or len(args) != 1 or args[0][2] != "topology"
                or not _graph_shape(args[0][1], types)):
            raise ValueError(f"OwnedAttachment shape is invalid: {operation['signature']}")
    if action == "non_owning_update":
        args = operation["args"]
        if (not receiver or not _graph_type(receiver, types) or result != receiver or len(args) != 1
                or _graph_shape(args[0][1], types)):
            raise ValueError(f"NonOwningUpdate shape is invalid: {operation['signature']}")
    if action == "relation_constructor":
        args = operation["args"]
        if (receiver is not None or result != "Relation" or not args
                or args[0] != ("key", "identifier", "topology")
                or not any(axis == "topology" and _graph_shape(value, types)
                           for _, value, axis in args[1:])):
            raise ValueError(f"RelationConstructor shape is invalid: {operation['signature']}")
    if action == "entity_constructor" and (
            receiver is not None or not graph or not operation["args"]
            or operation["args"][0] != ("key", "identifier", "topology")):
        raise ValueError(f"EntityConstructor shape is invalid: {operation['signature']}")
    lowering = semantics["temporal_lowering"]
    if lowering is None:
        return
    expected_result, expected_operands = TEMPORAL_SHAPES[lowering]
    actual_operands = tuple(value for _, value, _ in operation["args"])
    if (
        expected_result != result
        or expected_operands != actual_operands
        or semantics["max_stage"] != "temporal"
    ):
        raise ValueError(f"temporal lowering shape is invalid: {operation['signature']}")
    if any(axis != "leaf" for _, _, axis in operation["args"]):
        raise ValueError(f"temporal lowering requires leaf operands: {operation['signature']}")


def _graph_type(value, types):
    return value in types and types[value]["classification"] in GRAPH_CLASSES


def _graph_shape(value, types):
    inner = value[5:-1] if value.startswith("list<") else value
    return _graph_type(inner, types)
