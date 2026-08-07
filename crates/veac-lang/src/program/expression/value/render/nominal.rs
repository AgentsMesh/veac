use super::super::{EnumValue, StructValue, Value};
use crate::program::TypeDefinitionKind;

pub(super) fn structure(value: &StructValue, render: fn(&Value) -> String) -> String {
    let TypeDefinitionKind::Struct(layout) = value.definition().kind() else {
        unreachable!("verified nominal struct value retains a struct definition")
    };
    fields(
        value.definition().declared_name(),
        layout.fields(),
        value.fields(),
        render,
    )
}

pub(super) fn enumeration(value: &EnumValue, render: fn(&Value) -> String) -> String {
    let TypeDefinitionKind::Enum(layout) = value.definition().kind() else {
        unreachable!("verified nominal enum value retains an enum definition")
    };
    let variant = &layout.variants()[value.variant().index()];
    let name = format!("{}.{}", value.definition().declared_name(), variant.name());
    fields(&name, variant.fields(), value.fields(), render)
}

fn fields(
    name: &str,
    layout: &[crate::program::FieldDefinition],
    values: &[Value],
    render: fn(&Value) -> String,
) -> String {
    if values.is_empty() {
        return if name.contains('.') {
            name.to_owned()
        } else {
            format!("{name} {{}}")
        };
    }
    let fields = layout
        .iter()
        .zip(values)
        .map(|(field, value)| format!("{}: {}", field.name(), render(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name} {{ {fields} }}")
}
