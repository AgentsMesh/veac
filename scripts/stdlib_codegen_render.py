"""Rust renderers for the executable standard-library v6 generator."""

from __future__ import annotations

from collections import defaultdict

from stdlib_codegen_rust import (
    CONTRACT_ROOT,
    HEADER,
    IR_OPCODE_ROOT,
    OP_ROOT,
    PRIMITIVE_RUST,
    TYPE_ROOT,
    family_groups,
    op_constant,
    pascal,
    snake_type,
)


def assign_type_opcodes(types, families):
    counters = defaultdict(int)
    output = {"Context": 0x0001}
    family_index = {name: index + 1 for index, name in enumerate(families)}
    for name, metadata in types.items():
        if name == "Context":
            continue
        family = metadata["family"]
        counters[family] += 1
        output[name] = family_index[family] * 0x1000 + counters[family]
    return output


def render_types(types, families):
    opcodes = assign_type_opcodes(types, families)
    groups = defaultdict(list)
    groups["context"].append("Context")
    for name, metadata in types.items():
        if name != "Context":
            groups[metadata["family"]].append(name)
    rendered = {}
    modules = []
    for family, names in groups.items():
        for chunk_index, start in enumerate(range(0, len(names), 70)):
            module = f"{family.replace('-', '_')}_{chunk_index:02d}"
            modules.append(module)
            lines = [HEADER.rstrip(), "use super::super::identity::{declare_domain_types, DomainTypeIdentity};",
                     "use super::super::{classification::DomainTypeClassification, DomainType};", "",
                     "declare_domain_types! {"]
            for name in names[start:start + 70]:
                classification = types[name]["classification"]
                lines.append(f'    {name} = 0x{opcodes[name]:04x} => "{name}", {classification};')
            lines.append("}")
            rendered[TYPE_ROOT / f"{module}.rs"] = "\n".join(lines) + "\n"
    rendered[TYPE_ROOT / "mod.rs"] = table_module(modules, "DomainTypeIdentity")
    return rendered


def assign_operation_opcodes(operations, families):
    counters = defaultdict(int)
    family_index = {name: index + 1 for index, name in enumerate(families)}
    values = {}
    for operation in operations:
        family = operation["family"]
        counters[family] += 1
        values[operation["signature"]] = family_index[family] * 0x1000 + counters[family]
    return values


def render_operation_ids(operations, families):
    opcodes = assign_operation_opcodes(operations, families)
    groups = family_groups(operations)
    rendered = {}
    modules = []
    for family in families:
        values = groups[family]
        for chunk_index, start in enumerate(range(0, len(values), 70)):
            module = f"{family.replace('-', '_')}_{chunk_index:02d}"
            modules.append(module)
            lines = [HEADER.rstrip(), "use super::super::identity::{declare_operations, OperationIdentity};",
                     "use super::super::DomainOperationId;", "",
                     f"declare_operations!({pascal(family)} {{"]
            for operation in values[start:start + 70]:
                constant = op_constant(operation)
                opcode = opcodes[operation["signature"]]
                canonical = canonical_name(operation)
                lines.append(f'    {constant} = 0x{opcode:04x} => "{canonical}";')
            lines.append("});")
            rendered[OP_ROOT / f"{module}.rs"] = "\n".join(lines) + "\n"
    rendered[OP_ROOT / "mod.rs"] = table_module(modules, "OperationIdentity")
    return rendered


def render_ir_opcodes(operations, families):
    values = assign_operation_opcodes(operations, families)
    grouped = family_groups(operations)
    ranges = []
    for family in families:
        opcodes = sorted(values[value["signature"]] for value in grouped[family])
        if opcodes != list(range(opcodes[0], opcodes[-1] + 1)):
            raise ValueError(f"non-contiguous canonical opcode family: {family}")
        ranges.append(f"0x{opcodes[0]:04x}..=0x{opcodes[-1]:04x}")
    lines = [HEADER.rstrip(), "pub(super) const fn is_current(opcode: u16) -> bool {", "    matches!(", "        opcode,"]
    lines.extend(f"        {value}" + (" |" if index + 1 < len(ranges) else "")
                 for index, value in enumerate(ranges))
    lines.extend(["    )", "}"])
    return {IR_OPCODE_ROOT / "mod.rs": "\n".join(lines) + "\n"}


def canonical_name(operation):
    receiver = operation["receiver"]
    return f"{snake_type(receiver)}_{operation['name']}" if receiver else operation["name"]


def table_module(modules, identity):
    lines = [HEADER.rstrip()] + [f"mod {module};" for module in modules]
    lines += ["", f"pub(super) const COUNT: usize = {' + '.join(f'{m}::IDENTITIES.len()' for m in modules)};",
              f"pub(super) const TABLES: [&[super::identity::{identity}]; {len(modules)}] = ["]
    lines += [f"    {module}::IDENTITIES," for module in modules]
    lines.append("];\n")
    return "\n".join(lines)


def render_contracts(operations, action, axis):
    rendered = {}
    modules = []
    for family, values in family_groups(operations).items():
        for chunk_index, start in enumerate(range(0, len(values), 8)):
            module = f"{family.replace('-', '_')}_{chunk_index:02d}"
            modules.append(module)
            chunk = values[start:start + 8]
            uses_primitive = any("PrimitiveType" in shape(value)
                                 for operation in chunk for _, value in operation["args"])
            lines = contract_header(uses_primitive)
            for operation in chunk:
                lines.extend(contract_arm(operation, action, axis))
            lines += ["        _ => return None,", "    })", "}"]
            rendered[CONTRACT_ROOT / f"{module}.rs"] = "\n".join(lines) + "\n"
    lines = [HEADER.rstrip()] + [f"mod {module};" for module in modules]
    lines += ["", "use crate::program::domain_system::{DomainOperationContract, DomainOperationId};", "",
              "const LOOKUPS: &[fn(DomainOperationId) -> Option<DomainOperationContract>] = &["]
    lines += [f"    {module}::contract," for module in modules]
    lines += ["];", "", "pub(super) fn contract(id: DomainOperationId) -> DomainOperationContract {",
              "    LOOKUPS.iter().find_map(|lookup| lookup(id))",
              "        .expect(\"closed v6 operation has a generated contract\")", "}"]
    rendered[CONTRACT_ROOT / "mod.rs"] = "\n".join(lines) + "\n"
    return rendered


def contract_header(uses_primitive):
    lines = [HEADER.rstrip(), "use super::super::builders::*;",
            "use crate::program::domain_system::{", "    DomainOperationContract as Contract, DomainOperationId as Op,",
            "    DomainRuntimeAction as Action, DomainType as Type,", "};"]
    if uses_primitive:
        lines.append("use crate::program::expression::PrimitiveType;")
    return lines + ["", "pub(super) fn contract(id: Op) -> Option<Contract> {", "    Some(match id {"]


def contract_arm(operation, action, axis):
    constant = op_constant(operation)
    exposure = (f'method(Type::{operation["receiver"]}, "{operation["name"]}")'
                if operation["receiver"] else f'free("{operation["name"]}")')
    args = list(operation["args"])
    if operation["receiver"]:
        args.insert(0, (snake_type(operation["receiver"]), operation["receiver"]))
    lines = [f"        Op::{constant} => build(", "            id,", f"            {exposure},", "            vec!["]
    for index, (name, value) in enumerate(args):
        lines.append(f'                {axis(operation, index, name, value).lower()}("{name}", {shape(value)}),')
    lines += ["            ],", f"            Type::{operation['result']},",
              f"            Action::{action(operation)},", "        ),"]
    return lines


def shape(value):
    listed = value.startswith("list<")
    inner = value[5:-1] if listed else value
    if inner in PRIMITIVE_RUST:
        function = "primitive_list" if listed else "primitive"
        return f"{function}(PrimitiveType::{PRIMITIVE_RUST[inner]})"
    function = "domain_list" if listed else "domain"
    return f"{function}(Type::{inner})"


def render_all(operations, types, action, axis):
    families = list(dict.fromkeys(operation["family"] for operation in operations))
    rendered = {}
    rendered.update(render_types(types, families))
    rendered.update(render_operation_ids(operations, families))
    rendered.update(render_ir_opcodes(operations, families))
    rendered.update(render_contracts(operations, action, axis))
    oversized = [(path, text.count("\n")) for path, text in rendered.items() if text.count("\n") >= 200]
    if oversized:
        raise ValueError(f"generated files exceed the 199-line limit: {oversized}")
    return rendered
