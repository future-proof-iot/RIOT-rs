// TODO: document default values (which may be platform-dependent)
// TODO: document valid values
/// Runs the function decorated with this attribute macro as a separate thread.
///
/// # Parameters
///
/// - `autostart`: (*mandatory*) autostart the thread.
/// - `stacksize`: (*optional*) the size of the stack allocated to the thread (in bytes).
/// - `priority`: (*optional*) the thread's priority.
///
/// # Examples
///
/// This starts a thread with default values:
///
/// ```ignore
/// #[riot_rs::thread]
/// fn print_hello_world() {
///     println!("Hello world!");
/// }
/// ```
///
/// This starts a thread with a stack size of 1024 bytes and a priority of 2:
///
/// ```ignore
/// // `stacksize` and `priority` can be arbitrary expressions.
/// #[riot_rs::thread(stacksize = 1024, priority = 2)]
/// fn print_hello_world() {
///     println!("Hello world!");
/// }
/// ```
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn thread(args: TokenStream, item: TokenStream) -> TokenStream {
    #[allow(clippy::wildcard_imports)]
    use thread::*;

    use quote::quote;

    use crate::utils::find_crate;

    let mut attrs = Attributes::default();
    let thread_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with thread_parser);

    assert!(
        attrs.autostart,
        "the `autostart` parameter must be provided",
    );

    let thread_function = syn::parse_macro_input!(item as syn::ItemFn);

    let no_mangle_attr = if attrs.no_mangle {
        quote! {#[no_mangle]}
    } else {
        quote! {}
    };

    let fn_name = thread_function.sig.ident.clone();
    let Parameters {
        stack_size,
        priority,
    } = Parameters::from(attrs);

    let thread_crate = {
        match (find_crate("riot-rs"), find_crate("riot-rs-threads")) {
            (Some(riot_rs), _) => quote! { #riot_rs::thread },
            (None, Some(riot_rs_threads)) => quote! { #riot_rs_threads },
            _ => panic!(r#"neither "riot-rs" nor "riot-rs-threads" found in dependencies!"#),
        }
    };

    let expanded = quote! {
        #no_mangle_attr
        #thread_function

        #thread_crate::autostart_thread!(#fn_name, stacksize = #stack_size, priority = #priority);
    };

    TokenStream::from(expanded)
}

mod thread {
    pub struct Parameters {
        pub stack_size: syn::Expr,
        pub priority: syn::Expr,
    }

    impl Default for Parameters {
        fn default() -> Self {
            // TODO: proper values
            Self {
                stack_size: syn::parse_quote!{ 2048 },
                priority: syn::parse_quote!{ 1 },
            }
        }
    }

    impl From<Attributes> for Parameters {
        fn from(attrs: Attributes) -> Self {
            let default = Self::default();

            let stack_size = attrs.stack_size.unwrap_or(default.stack_size);
            let priority = attrs.priority.unwrap_or(default.priority);

            Self {
                stack_size,
                priority,
            }
        }
    }

    #[derive(Default)]
    pub struct Attributes {
        pub autostart: bool,
        pub stack_size: Option<syn::Expr>,
        pub priority: Option<syn::Expr>,
        pub no_mangle: bool,
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

            if meta.path.is_ident("no_mangle") {
                self.no_mangle = true;
                return Ok(());
            }

            Err(meta.error("unsupported parameter"))
        }
    }
}
