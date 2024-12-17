/// This macro allows to obtain peripherals from the one listed in the `peripherals` module
/// exported by this crate.
///
/// It makes sense to use this macro multiple times, coupled with conditional compilation (using
/// the [`cfg`
/// attribute](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-attribute)),
/// to define different setups for different boards.
///
/// # Note
///
/// The [`define_peripherals!`](crate::define_peripherals!) macro expects the
/// `ariel_os::hal::peripherals` module to be in scope.
///
// Inspired by https://github.com/adamgreig/assign-resources/tree/94ad10e2729afdf0fd5a77cd12e68409a982f58a
// under MIT license
#[macro_export]
macro_rules! define_peripherals {
    (
        $(#[$outer:meta])*
        $peripherals:ident {
            $(
                $(#[$inner:meta])*
                $peripheral_name:ident : $peripheral_field:ident $(=$peripheral_alias:ident)?
            ),*
            $(,)?
        }
    ) => {
        #[allow(dead_code,non_snake_case)]
        $(#[$outer])*
        pub struct $peripherals {
            $(
                $(#[$inner])*
                pub $peripheral_name: peripherals::$peripheral_field
            ),*
        }

        $($(
            #[allow(missing_docs, non_camel_case_types)]
            pub type $peripheral_alias = peripherals::$peripheral_field;
        )?)*

        impl $crate::TakePeripherals<$peripherals> for &mut $crate::OptionalPeripherals {
            fn take_peripherals(&mut self) -> $peripherals {
                $peripherals {
                    $(
                        $(#[$inner])*
                        $peripheral_name: self.$peripheral_field.take().unwrap()
                    ),*
                }
            }
        }
    }
}

/// This macro allows to group peripheral structs defined with
/// [`define_peripherals!`](crate::define_peripherals!) into a single peripheral struct.
#[macro_export]
macro_rules! group_peripherals {
    (
        $(#[$outer:meta])*
        $group:ident {
            $(
                $(#[$inner:meta])*
                $peripheral_name:ident : $peripherals:ident
            ),*
            $(,)?
        }
    ) => {
        #[allow(dead_code,non_snake_case)]
        $(#[$outer])*
        pub struct $group {
            $(
                $(#[$inner])*
                pub $peripheral_name: $peripherals
            ),*
        }

        impl $crate::TakePeripherals<$group> for &mut $crate::OptionalPeripherals {
            fn take_peripherals(&mut self) -> $group {
                $group {
                    $(
                        $(#[$inner])*
                        $peripheral_name: self.take_peripherals()
                    ),*
                }
            }
        }
    }
}

#[doc(hidden)]
pub trait TakePeripherals<T> {
    fn take_peripherals(&mut self) -> T;
}
