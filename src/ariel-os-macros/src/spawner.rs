/// Registers a non-async function for autostart.
///
/// The function is provided with:
///
/// - a `Spawner` as first parameter,
/// - a peripheral struct, as optional second parameter.
///
/// The peripheral struct must be defined with the `ariel_os::hal::define_peripherals!` macro.
///
/// See [`macro@task`] to use a long-lived async function instead.
///
/// # Parameters
///
/// - `autostart`: (*mandatory*) run the task at startup.
/// - `peripherals`: (*optional*) provide the function with a peripheral struct as the second
///     parameter.
///
/// # Examples
///
/// ```ignore
/// use ariel_os::asynch::Spawner;
///
/// #[ariel_os::spawner(autostart, peripherals)]
/// fn spawner(spawner: Spawner, peripherals: /* your peripheral type */) {}
/// ```
///
/// See Ariel OS examples for more.
///
/// # Panics
///
/// This macro panics when the `ariel-os` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn spawner(args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use syn::{spanned::Spanned, Error};

    #[allow(clippy::wildcard_imports)]
    use spawner::*;

    let mut attrs = Attributes::default();
    let spawner_attr_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with spawner_attr_parser);

    assert!(
        attrs.autostart,
        "the `{AUTOSTART_PARAM}` parameter must be provided",
    );

    let spawner_function = syn::parse_macro_input!(item as syn::ItemFn);
    let spawner_function_name = &spawner_function.sig.ident;
    let is_async = spawner_function.sig.asyncness.is_some();

    assert!(
        !is_async,
        "spawner functions cannot be async, consider using `task` instead",
    );

    let param_count = spawner_function.sig.inputs.len();
    if param_count == 0 {
        let span = spawner_function.sig.span();
        let error = Error::new(span, "`spawner: Spawner` function argument missing");
        return error.to_compile_error().into();
    } else if param_count == 2 && !attrs.peripherals {
        let span = proc_macro2::Span::call_site();
        let mut error = Error::new(span, "`peripherals` macro parameter missing here ...");

        error.combine(Error::new(
            spawner_function.sig.inputs.span(),
            "... because this function has a second parameter",
        ));

        return error.to_compile_error().into();
    }

    let ariel_os_crate = utils::ariel_os_crate();

    let new_function_name = format_ident!("__start_{spawner_function_name}");

    let peripheral_param = if attrs.peripherals {
        quote! {, peripherals.take_peripherals()}
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[allow(non_snake_case)]
        #[#ariel_os_crate::reexports::linkme::distributed_slice(#ariel_os_crate::EMBASSY_TASKS)]
        #[linkme(crate = #ariel_os_crate::reexports::linkme)]
        fn #new_function_name(
            spawner: #ariel_os_crate::asynch::Spawner,
            mut peripherals: &mut #ariel_os_crate::hal::OptionalPeripherals,
        ) {
            use #ariel_os_crate::hal::define_peripherals::TakePeripherals;
            #spawner_function_name(spawner #peripheral_param);
        }

        #spawner_function
    };

    TokenStream::from(expanded)
}

mod spawner {
    pub const AUTOSTART_PARAM: &str = "autostart";
    pub const PERIPHERALS_PARAM: &str = "peripherals";

    #[derive(Debug, Default)]
    pub struct Attributes {
        pub autostart: bool,
        pub peripherals: bool,
    }

    impl Attributes {
        #[allow(clippy::missing_errors_doc)]
        #[allow(clippy::unnecessary_wraps)]
        pub fn parse(&mut self, attr: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
            if attr.path.is_ident(AUTOSTART_PARAM) {
                self.autostart = true;
            } else if attr.path.is_ident(PERIPHERALS_PARAM) {
                self.peripherals = true;
            }

            Ok(())
        }
    }
}
