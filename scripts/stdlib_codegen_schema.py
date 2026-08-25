"""Closed scalar vocabularies for Domain opspec v3."""

CLASSIFICATIONS = {
    "container": "Container",
    "graph_entity": "GraphEntity",
    "topology_value": "TopologyValue",
    "leaf_value": "LeafValue",
}
AXES = {"topology": "topology", "leaf": "leaf"}
INSTRUCTIONS = {
    "domain_construct": "DomainConstruct",
    "graph_emit": "GraphEmit",
}
RUNTIME_ACTIONS = {
    "description": "Description",
    "entity_constructor": "EntityConstructor",
    "owned_attachment": "OwnedAttachment",
    "non_owning_update": "NonOwningUpdate",
    "project_entry": "ProjectEntry",
    "relation_constructor": "RelationConstructor",
}
EFFECTS = {"pure": "Pure", "graph_emit": "GraphEmit"}
STAGES = {"build": "Build", "temporal": "Temporal"}
TEMPORAL_LOWERINGS = {
    "compose_vector": "ComposeVector",
    "compose_point": "ComposePoint",
    "compose_rect": "ComposeRect",
}


def closed_enum(value, values, label):
    if not isinstance(value, str) or value not in values:
        raise ValueError(f"invalid {label}: {value!r}")
    return value


def opcode(value, label):
    if type(value) is not int or not 0 < value <= 0xffff:
        raise ValueError(f"{label} must be an integer in 1..=65535; found {value!r}")
    return value
