use super::super::use_macro::define_control_uses;
use crate::program::expression::TypeConstructor;

define_control_uses! {
    LIST_TYPE_CONSTRUCTOR => [TypeConstructor::List.as_str()] @ StructuralTypeConstructor : KindDiscriminator;
    MAP_TYPE_CONSTRUCTOR => [TypeConstructor::Map.as_str()] @ StructuralTypeConstructor : KindDiscriminator;
    RANGE_TYPE_CONSTRUCTOR => [TypeConstructor::Range.as_str()] @ StructuralTypeConstructor : KindDiscriminator;
    FUNCTION_TYPE_CONSTRUCTOR => [TypeConstructor::Function.as_str()] @ StructuralTypeConstructor : KindDiscriminator;
}
