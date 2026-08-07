"""Closed effect, ownership, and stage-axis policy for stdlib v6 generation."""

PRIMITIVES = {
    "int", "scalar", "time", "length", "percent", "angle", "text", "color", "bool",
    "identifier",
}
GRAPH_TYPES = {
    "Project", "Sequence", "Layer", "Item", "Resource", "Relation", "MulticamGroup",
    "Apply", "Annotation", "Delivery",
}
OWNED_METHODS = {
    ("Project", "with_resource"),
    ("Project", "with_sequence"),
    ("Project", "with_multicam_group"),
    ("Project", "with_annotation"),
    ("Project", "with_delivery"),
    ("Sequence", "with_layer"),
    ("Sequence", "with_relation"),
    ("Sequence", "with_apply"),
    ("Layer", "with_item"),
    ("Project", "with_resources"),
    ("Project", "with_sequences"),
    ("Project", "with_multicam_groups"),
    ("Project", "with_annotations"),
    ("Project", "with_deliveries"),
    ("Sequence", "with_layers"),
    ("Sequence", "with_relations"),
    ("Sequence", "with_applies"),
    ("Layer", "with_items"),
}
STATIC_TYPES = GRAPH_TYPES | {
    "Context", "Canvas", "FrameRate", "Source", "ContentIdentity", "ProjectSettings",
    "SequenceSettings",
    "TrackEditing", "TrackPlayback", "TrackIsolation", "TrackRouting", "TrackState",
    "ResourceLocation", "StreamChoice", "StreamIntent", "SourceTiming", "SourceMapping",
    "SourceTimeMap", "SourceTimeSegment", "Generator", "Deliverable", "DeliverableTarget",
    "VideoDelivery", "VideoOutput", "AudioOutput", "CaptionOutput",
    "PluginEffectDescriptor",
}
TOPOLOGY_NAMES = {
    "key", "kind", "state", "record", "source", "timing", "target", "order",
    "placement", "routing", "resource", "sequence", "layer", "item", "from", "to",
    "members", "video", "audio", "sidechain", "angles", "switches", "artifacts",
    "owner", "identity", "location", "stream", "contract", "scope", "enabled",
}


def inner_type(value: str) -> str:
    return value[5:-1] if value.startswith("list<") else value


def action(operation):
    surface = (operation["receiver"], operation["name"])
    if surface == ("Project", "entry"):
        return "ProjectEntry"
    if surface in OWNED_METHODS:
        return "OwnedAttachment"
    if operation["receiver"]:
        return "NonOwningUpdate"
    if operation["result"] == "Relation":
        return "RelationConstructor"
    if operation["result"] in GRAPH_TYPES:
        return "EntityConstructor"
    return "Description"


def axis(operation, index, name, value):
    if operation["receiver"] and index == 0:
        return "Topology"
    if operation["name"] in {"canvas", "frame_rate"}:
        return "Topology"
    inner = inner_type(value)
    if action(operation) != "Description" and name in TOPOLOGY_NAMES:
        return "Topology"
    dynamic_primitives = {"scalar", "time", "length", "percent", "angle", "color"}
    if inner in GRAPH_TYPES or inner in STATIC_TYPES or inner in PRIMITIVES - dynamic_primitives:
        return "Topology"
    return "Leaf"
