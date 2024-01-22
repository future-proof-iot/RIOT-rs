/// This macro allows to extract the specified peripherals from `OptionalPeripherals` for use in an
/// application.
///
/// The generated struct provides a `take_from()` method for extracting the specified peripherals
/// from `OptionalPeripherals`.
///
/// The `define_peripherals!` macro expects a `peripherals` module to be in scope, where the
/// peripheral types should come from.
///
/// It makes sense to use this macro multiple times, coupled with conditional compilation (using
/// the [`cfg`
/// attribute](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-attribute)),
/// to define different setups for different boards.
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
                $peripheral_name:ident : $peripheral_field:ident $(=$peripheral_alias:ident)?),*
            $(,)?
        }
    ) => {
        #[allow(dead_code,non_snake_case,missing_docs)]
        $(#[$outer])*
        pub struct $peripherals {
            $(
                $(#[$inner])*
                pub $peripheral_name: peripherals::$peripheral_field
            ),*
        }

        $($($(
            #[allow(missing_docs)]
            pub type $peripheral_alias = peripherals::$peripheral_field;
        )?)*)*

        impl $peripherals {
            pub fn take_from(
                opt_peripherals: &mut $crate::arch::OptionalPeripherals
            ) -> Result<Self, $crate::define_peripherals::DefinePeripheralsError> {
                Ok(Self {
                    $($peripheral_name: opt_peripherals.$peripheral_field
                        .take()
                        .ok_or($crate::define_peripherals::DefinePeripheralsError::TakingPeripheral)?
                    ),*
                })
            }
        }
    }
}

pub enum DefinePeripheralsError {
    TakingPeripheral,
}
