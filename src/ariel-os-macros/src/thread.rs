// TODO: document default values (which may be platform-dependent)
// TODO: document valid values
/// Runs the function decorated with this attribute macro as a separate thread.
///
/// # Parameters
///
/// - `autostart`: (*mandatory*) autostart the thread.
/// - `stacksize`: (*optional*) the size of the stack allocated to the thread (in bytes).
/// - `priority`: (*optional*) the thread's priority.
/// - `no_wait`: (*optional*) don't wait for system initialization to be finished
///              before starting the thread.
///
/// # Examples
///
/// This starts a thread with default values:
///
/// ```ignore
/// #[ariel_os::thread(autostart)]
/// fn print_hello_world() {
///     println!("Hello world!");
/// }
/// ```
///
/// This starts a thread with a stack size of 1024 bytes and a priority of 2:
///
/// ```ignore
/// // `stacksize` and `priority` can be arbitrary expressions.
/// #[ariel_os::thread(autostart, stacksize = 1024, priority = 2)]
/// fn print_hello_world() {
///     println!("Hello world!");
/// }
/// ```
///
/// # Panics
///
/// This macro panics when the `ariel-os` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn thread(args: TokenStream, item: TokenStream) -> TokenStream {
    #[allow(clippy::wildcard_imports)]
    use thread::*;

    use quote::{format_ident, quote};

    use crate::utils::find_crate;

    let mut attrs = Attributes::default();
    let thread_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with thread_parser);

    assert!(
        attrs.autostart,
        "the `autostart` parameter must be provided",
    );

    let thread_function = syn::parse_macro_input!(item as syn::ItemFn);

    let thread_crate = {
        match (find_crate("ariel-os"), find_crate("ariel-os-threads")) {
            (Some(ariel_os), _) => quote! { #ariel_os::thread },
            (None, Some(ariel_os_threads)) => quote! { #ariel_os_threads },
            _ => panic!(r#"neither "ariel-os" nor "ariel-os-threads" found in dependencies!"#),
        }
    };

    let maybe_wait_for_start_event = if attrs.no_wait {
        quote! {}
    } else {
        quote! {#thread_crate::events::THREAD_START_EVENT.wait();}
    };

    let fn_name = thread_function.sig.ident.clone();
    let trampoline_function_name = format_ident!("__{fn_name}_trampoline");

    let Parameters {
        stack_size,
        priority,
        affinity,
    } = Parameters::from(attrs);

    let expanded = quote! {
        #[inline(always)]
        #thread_function

        fn #trampoline_function_name() {
            #maybe_wait_for_start_event;
            #fn_name()
        }

        #thread_crate::autostart_thread!(#trampoline_function_name, stacksize = #stack_size, priority = #priority, affinity = #affinity);
    };

    TokenStream::from(expanded)
}

mod thread {
    pub struct Parameters {
        pub stack_size: syn::Expr,
        pub priority: syn::Expr,
        pub affinity: syn::Expr,
    }

    impl Default for Parameters {
        fn default() -> Self {
            // TODO: proper values
            Self {
                stack_size: syn::parse_quote! { 2048 },
                priority: syn::parse_quote! { 1 },
                affinity: syn::parse_quote! { None },
            }
        }
    }

    impl From<Attributes> for Parameters {
        fn from(attrs: Attributes) -> Self {
            let default = Self::default();

            let stack_size = attrs.stack_size.unwrap_or(default.stack_size);
            let priority = attrs.priority.unwrap_or(default.priority);
            let affinity = attrs
                .affinity
                .map(|expr| syn::parse_quote! { Some(#expr) })
                .unwrap_or(default.affinity);

            Self {
                stack_size,
                priority,
                affinity,
            }
        }
    }

    #[derive(Default)]
    pub struct Attributes {
        pub autostart: bool,
        pub stack_size: Option<syn::Expr>,
        pub priority: Option<syn::Expr>,
        pub affinity: Option<syn::Expr>,
        pub no_wait: bool,
    }

    impl Attributes {
        /// Parse macro attributes.
        ///
        /// # Errors
        ///
        /// Returns an error when an unsupported parameter is found.
        pub fn parse(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
            if meta.path.is_ident("autostart") {
                self.autostart = true;
                return Ok(());
            }

            if meta.path.is_ident("stacksize") {
                self.stack_size = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("priority") {
                self.priority = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("affinity") {
                self.affinity = Some(meta.value()?.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("no_wait") {
                self.no_wait = true;
                return Ok(());
            }

            Err(meta.error("unsupported parameter"))
        }
    }
}
