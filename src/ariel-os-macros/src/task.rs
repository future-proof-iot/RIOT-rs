/// Defines an async task and optionally registers it for autostart.
///
/// If this function is only used to spawn other tasks before returning, consider using
/// [`macro@spawner`] instead, to avoid statically allocating this transient async function as an
/// `embassy_executor::task`.
///
/// # Parameters
///
/// - `autostart`: (*optional*) run the task at startup; required to use `peripherals` and/or
///     hooks.
///     - `peripherals`: (*optional*) provide the function with a peripheral struct as the first
///         parameter.
///         The `peripherals` parameter can only be used on `autostart` tasks.
///         The peripheral struct must be defined with the `ariel_os::define_peripherals!` macro.
///     - hooks: (*optional*) available hooks are:
///         - `usb_builder_hook`: when present, the macro will define a static `USB_BUILDER_HOOK`
///           of type `UsbBuilderHook`, allowing to access and modify the system-provided
///           `embassy_usb::Builder` through `Delegate::with()`, *before* it is built by the system.
/// - `pool_size`: (*optional*) set the maximum number of concurrent tasks that can be spawned for
///     the function (defaults to `1`).
///     Cannot be used on `autostart` tasks.
///
/// # Examples
///
/// ```ignore
/// use ariel_os::usb::UsbBuilderHook;
///
/// #[ariel_os::task(autostart, peripherals, usb_builder_hook)]
/// async fn task(peripherals: /* your peripheral type */) {}
/// ```
///
/// See Ariel OS examples for more.
///
/// # Panics
///
/// This macro panics when the `ariel-os` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};

    #[allow(clippy::wildcard_imports)]
    use task::*;

    let mut attrs = Attributes::default();
    let task_attr_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with task_attr_parser);

    let task_function = syn::parse_macro_input!(item as syn::ItemFn);
    let task_function_name = &task_function.sig.ident;
    let is_async = task_function.sig.asyncness.is_some();

    assert!(is_async, "the function must be async");

    if attrs.autostart {
        assert!(
            attrs.pool_size.is_none(),
            "pool size cannot be set on an `{AUTOSTART_PARAM}` task",
        );

        if !attrs.peripherals {
            let param_count = task_function.sig.inputs.len();
            assert!(
                param_count == 0,
                "to provide this function with peripherals, use the `{PERIPHERALS_PARAM}` macro parameter",
            );
        }
    } else {
        assert!(
            !attrs.peripherals,
            "the task must be `{AUTOSTART_PARAM}` to receive peripherals"
        );

        assert!(
            attrs.hooks.is_empty(),
            "the task must be `{AUTOSTART_PARAM}` to instantiate hooks",
        );
    }

    // TODO: forbid generics on the function

    let ariel_os_crate = utils::ariel_os_crate();

    let expanded = if attrs.autostart {
        let peripheral_param = if attrs.peripherals {
            quote! {peripherals.take_peripherals()}
        } else {
            quote! {}
        };

        let hooks = Hook::hook_definitions();
        let delegates = task::generate_delegates(&ariel_os_crate, &hooks, &attrs);

        let new_function_name = format_ident!("__start_{task_function_name}");

        quote! {
            #delegates

            #[allow(non_snake_case)]
            #[#ariel_os_crate::reexports::linkme::distributed_slice(#ariel_os_crate::EMBASSY_TASKS)]
            #[linkme(crate = #ariel_os_crate::reexports::linkme)]
            fn #new_function_name(
                spawner: #ariel_os_crate::asynch::Spawner,
                mut peripherals: &mut #ariel_os_crate::hal::OptionalPeripherals,
            ) {
                use #ariel_os_crate::define_peripherals::TakePeripherals;
                let task = #task_function_name(#peripheral_param);
                spawner.spawn(task).unwrap();
            }

            #[#ariel_os_crate::reexports::embassy_executor::task(embassy_executor = #ariel_os_crate::reexports::embassy_executor)]
            #task_function
        }
    } else {
        let pool_size = attrs.pool_size.unwrap_or_else(|| syn::parse_quote! { 1 });

        quote! {
            #[#ariel_os_crate::reexports::embassy_executor::task(pool_size = #pool_size, embassy_executor = #ariel_os_crate::reexports::embassy_executor)]
            #task_function
        }
    };

    TokenStream::from(expanded)
}

// Define these types in a module to avoid polluting the crate's namespace, as this file is
// `included!` in the crate's root.
mod task {
    pub const AUTOSTART_PARAM: &str = "autostart";
    pub const PERIPHERALS_PARAM: &str = "peripherals";
    pub const POOL_SIZE_PARAM: &str = "pool_size";

    #[derive(Debug, Default)]
    pub struct Attributes {
        pub autostart: bool,
        pub peripherals: bool,
        pub pool_size: Option<syn::Expr>,
        pub hooks: Vec<Hook>,
    }

    impl Attributes {
        #[allow(clippy::missing_errors_doc)]
        pub fn parse(&mut self, attr: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
            if attr.path.is_ident(AUTOSTART_PARAM) {
                self.autostart = true;
                return Ok(());
            }

            if attr.path.is_ident(PERIPHERALS_PARAM) {
                self.peripherals = true;
                return Ok(());
            }

            if attr.path.is_ident(POOL_SIZE_PARAM) {
                let value = attr.value()?;
                self.pool_size = Some(value.parse()?);
                return Ok(());
            }

            // The order in which hooks are passed to the macro is enforced here
            for HookDefinition { kind, .. } in Hook::hook_definitions() {
                if attr.path.is_ident(kind.param_name()) {
                    self.hooks.push(kind);
                    return Ok(());
                }
            }

            let supported_hooks = Hook::format_list();
            Err(attr.error(format!(
                "unsupported parameter (`{AUTOSTART_PARAM}`, `{PERIPHERALS_PARAM}`, `{POOL_SIZE_PARAM}`, and hooks {supported_hooks} are supported)"
            )))
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, enum_iterator::Sequence)]
    pub enum Hook {
        UsbBuilder,
    }

    impl Hook {
        pub fn param_name(&self) -> &'static str {
            match self {
                Self::UsbBuilder => "usb_builder_hook",
            }
        }

        pub fn type_name(&self) -> &'static str {
            match self {
                Self::UsbBuilder => "UsbBuilderHook",
            }
        }

        pub fn delegate_ident(&self) -> String {
            self.param_name().to_uppercase()
        }

        fn format_list() -> String {
            enum_iterator::all::<Self>()
                .map(|h| format!("`{}`", h.param_name()))
                .collect::<Vec<_>>()
                .join(", ")
        }

        pub fn hook_definitions() -> [HookDefinition; 1] {
            use quote::quote;

            let ariel_os_crate = crate::utils::ariel_os_crate();

            // New hooks need to be defined here, in the order they are run during system
            // initialization
            [HookDefinition {
                kind: Self::UsbBuilder,
                delegate_inner_type: quote! {#ariel_os_crate::usb::UsbBuilder},
                distributed_slice_type: quote! {#ariel_os_crate::usb::USB_BUILDER_HOOKS},
            }]
        }
    }

    #[derive(Debug)]
    pub struct HookDefinition {
        pub kind: Hook,
        pub delegate_inner_type: proc_macro2::TokenStream,
        pub distributed_slice_type: proc_macro2::TokenStream,
    }

    pub fn generate_delegates(
        ariel_os_crate: &syn::Ident,
        hooks: &[HookDefinition],
        attrs: &Attributes,
    ) -> proc_macro2::TokenStream {
        use quote::{format_ident, quote};

        let delegate_type = quote! {#ariel_os_crate::delegate::Delegate};

        let enabled_hooks = hooks.iter().filter(|hook| match hook.kind {
            Hook::UsbBuilder => attrs.hooks.iter().any(|h| *h == Hook::UsbBuilder),
        });

        // Instantiate a Delegate as a static and store a reference to it in the appropriate
        // distributed slice
        let delegates = enabled_hooks.clone().map(|hook| {
            let HookDefinition { kind, delegate_inner_type, distributed_slice_type } = hook;

            let delegate_ident = kind.delegate_ident();

            let type_name = format_ident!("{}", kind.type_name());
            let delegate_hook_ident = format_ident!("{delegate_ident}");
            let delegate_hook_ref_ident = format_ident!("{delegate_ident}_REF");

            // TODO: try to reduce namespace pollution
            quote! {
                static #delegate_hook_ident: #delegate_type<#delegate_inner_type> = #delegate_type::new();

                #[#ariel_os_crate::reexports::linkme::distributed_slice(#distributed_slice_type)]
                #[linkme(crate=#ariel_os_crate::reexports::linkme)]
                    static #delegate_hook_ref_ident: #type_name = &#delegate_hook_ident;
                }
            }
        );

        quote! {#(#delegates)*}
    }
}
