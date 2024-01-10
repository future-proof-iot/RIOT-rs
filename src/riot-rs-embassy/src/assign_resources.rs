/// Similarly to
/// [`assign_resources`](https://docs.rs/assign-resources/latest/assign_resources/macro.assign_resources.html),
/// this macro allows to extract the specified peripherals from `OptionalPeripherals` for use in an
/// application.
/// This implementation takes an extra first parameter, which is an identifier used to name the
/// resulting struct.
///
/// It makes sense to use this macro multiple times, coupled with conditional compilation (using
/// the [`cfg`
/// attribute](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-attribute)),
/// to define different setups for different boards.
///
/// Using the `assign_resources!` macro to define the peripherals to extract will generate another
/// macro, `split_resources!`, that allows to obtain the requested peripherals where needed (see
/// the original documentation for details).
/// The `assign_resources!` macro expects a `peripherals` module to be in scope, where the
/// peripheral types should come from.
///
/// `split_resources!` should be provided with an instance of `OptionalPeripherals` to extract
/// peripherals from.
/// When attempting to extract a peripheral from `OptionalPeripherals` that has already been
/// extracted, `split_resources!` will return from the function where it has been called in with
/// [`Err(AssigningResourcesError::ObtainingPeripheral)?`].
// Based on https://github.com/adamgreig/assign-resources/tree/94ad10e2729afdf0fd5a77cd12e68409a982f58a
// under MIT license
#[macro_export]
macro_rules! assign_resources {
    {
        $resources: ident,
        $(
            $(#[$outer:meta])*
            $group_name:ident : $group_struct:ident {
                $(
                    $(#[$inner:meta])*
                    $resource_name:ident : $resource_field:ident $(=$resource_alias:ident)?),*
                $(,)?
            }
            $(,)?
        )+
    } => {
        #[allow(dead_code,non_snake_case,missing_docs)]
        pub struct $resources {
            $(pub $group_name : $group_struct),*
        }
        $(
            #[allow(dead_code,non_snake_case)]
            $(#[$outer])*
            pub struct $group_struct {
                $(
                    $(#[$inner])*
                    pub $resource_name: peripherals::$resource_field
                ),*
            }
        )+


        $($($(
            #[allow(missing_docs)]
            pub type $resource_alias = peripherals::$resource_field;
        )?)*)*

        #[macro_export]
        /// `split_resources!` macro
        macro_rules! split_resources (
            ($p:ident) => {
                $resources {
                    $($group_name: $group_struct {
                        $($resource_name: $p.$resource_field
                            .take()
                            .ok_or($crate::assign_resources::AssigningResourcesError::ObtainingPeripheral)?),*
                    }),*
                }
            }
        );
    }
}

pub enum AssigningResourcesError {
    ObtainingPeripheral,
}
