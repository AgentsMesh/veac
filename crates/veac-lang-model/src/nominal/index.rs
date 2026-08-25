macro_rules! dense_index {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u16);

        impl $name {
            pub const fn new(value: u16) -> Self {
                Self(value)
            }

            pub const fn value(self) -> u16 {
                self.0
            }

            pub const fn index(self) -> usize {
                self.0 as usize
            }

            #[doc(hidden)]
            pub fn from_position(value: usize) -> Option<Self> {
                u16::try_from(value).ok().map(Self)
            }
        }
    };
}

dense_index!(FieldIndex);
dense_index!(VariantIndex);
