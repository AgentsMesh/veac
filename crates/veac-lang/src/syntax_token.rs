pub trait SyntaxToken: Copy + Eq + 'static {
    const ALL: &'static [Self];
    const TOKENS: &'static [&'static str];
    const ENTRIES: &'static [(&'static str, Self)];

    fn as_str(self) -> &'static str;
    fn parse(value: &str) -> Option<Self>;
}

macro_rules! impl_local_syntax_token_body {
    ($ty:ty, $($pattern:pat => $value:expr => $token:literal),+ $(,)?) => {
        impl $ty {
            pub const ALL: &'static [Self] = &[$($value),+];
            pub const TOKENS: &'static [&'static str] = &[$($token),+];
            pub const ENTRIES: &'static [(&'static str, Self)] = &[$(($token, $value)),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $($pattern => $token),+
                }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value {
                    $($token => Some($value),)+
                    _ => None,
                }
            }
        }

        impl $crate::SyntaxToken for $ty {
            const ALL: &'static [Self] = <$ty>::ALL;
            const TOKENS: &'static [&'static str] = <$ty>::TOKENS;
            const ENTRIES: &'static [(&'static str, Self)] = <$ty>::ENTRIES;

            fn as_str(self) -> &'static str {
                <$ty>::as_str(self)
            }

            fn parse(value: &str) -> Option<Self> {
                <$ty>::parse(value)
            }
        }
    };
}

macro_rules! impl_local_syntax_token_array_body {
    ($ty:ty, $($variant:path => $token:literal),+ $(,)?) => {
        impl $ty {
            pub const ALL: [Self; [$($token),+].len()] = [$($variant),+];
            pub const TOKENS: &'static [&'static str] = &[$($token),+];
            pub const ENTRIES: &'static [(&'static str, Self)] = &[$(($token, $variant)),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $($variant => $token),+
                }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value {
                    $($token => Some($variant),)+
                    _ => None,
                }
            }
        }

        impl $crate::SyntaxToken for $ty {
            const ALL: &'static [Self] = &<$ty>::ALL;
            const TOKENS: &'static [&'static str] = <$ty>::TOKENS;
            const ENTRIES: &'static [(&'static str, Self)] = <$ty>::ENTRIES;

            fn as_str(self) -> &'static str {
                <$ty>::as_str(self)
            }

            fn parse(value: &str) -> Option<Self> {
                <$ty>::parse(value)
            }
        }
    };
}

macro_rules! define_syntax_tokens {
    (array $(#[$meta:meta])* $vis:vis enum $name:ident {
        $($variant:ident => $token:literal),+ $(,)?
    }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis enum $name {
            $($variant),+
        }

        $crate::impl_local_syntax_token_array_body!(
            $name,
            $($name::$variant => $token),+
        );
    };
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($variant:ident => $token:literal),+ $(,)?
    }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis enum $name {
            $($variant),+
        }

        $crate::impl_local_syntax_tokens!($name, $($name::$variant => $token),+);
    };
}

macro_rules! impl_local_syntax_tokens {
    ($ty:ty, $($variant:path => $token:literal),+ $(,)?) => {
        $crate::impl_local_syntax_token_body!(
            $ty,
            $($variant => $variant => $token),+
        );
    };
}

macro_rules! impl_composite_syntax_tokens {
    ($ty:ty, $($pattern:pat => $value:expr => $token:literal),+ $(,)?) => {
        $crate::impl_local_syntax_token_body!(
            $ty,
            $($pattern => $value => $token),+
        );
    };
}

macro_rules! impl_syntax_tokens {
    ($ty:ty, $($variant:path => $token:literal),+ $(,)?) => {
        impl $crate::SyntaxToken for $ty {
            const ALL: &'static [Self] = &[$($variant),+];
            const TOKENS: &'static [&'static str] = &[$($token),+];
            const ENTRIES: &'static [(&'static str, Self)] = &[$(($token, $variant)),+];

            fn as_str(self) -> &'static str {
                match self {
                    $($variant => $token),+
                }
            }

            fn parse(value: &str) -> Option<Self> {
                match value {
                    $($token => Some($variant),)+
                    _ => None,
                }
            }
        }
    };
}

pub(crate) use {
    define_syntax_tokens, impl_composite_syntax_tokens, impl_local_syntax_token_array_body,
    impl_local_syntax_token_body, impl_local_syntax_tokens, impl_syntax_tokens,
};

#[cfg(test)]
mod tests;
