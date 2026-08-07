macro_rules! define_control_uses {
    (@emit $($name:ident, $spelling:expr, $position:ident, $role:ident;)+) => {
        $(
            pub(crate) const $name: crate::vocabulary::ControlUse =
                crate::vocabulary::ControlUse::new(
                    crate::vocabulary::ControlWord::new($spelling),
                    crate::vocabulary::GrammarPosition::$position,
                    crate::vocabulary::CanonicalRole::$role,
                );
        )+

        pub(crate) const ALL: &[crate::vocabulary::ControlUse] = &[$($name),+];
    };
    ($($name:ident => [$spelling:expr] @ $position:ident : $role:ident;)+) => {
        define_control_uses!(@emit $($name, $spelling, $position, $role;)+);
    };
    ($($name:ident => $spelling:literal @ $position:ident : $role:ident;)+) => {
        define_control_uses!(@emit $($name, $spelling, $position, $role;)+);
    };
}

pub(super) use define_control_uses;
