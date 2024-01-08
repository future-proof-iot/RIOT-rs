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
                        $($resource_name: $p.$resource_field.take().ok_or(1)?),*
                    }),*
                }
            }
        );
    }
}
