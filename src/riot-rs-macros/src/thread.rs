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

    let riot_rs_crate = utils::riot_rs_crate();

    let expanded = quote! {
        #no_mangle_attr
        #thread_function

        #riot_rs_crate::thread::autostart_thread!(#fn_name, stacksize = #stack_size, priority = #priority);
    };

    TokenStream::from(expanded)
}

mod thread {
    pub struct Parameters {
        pub stack_size: u64,
        pub priority: u8,
    }

    impl Default for Parameters {
        fn default() -> Self {
            // TODO: proper values
            Self {
                stack_size: 2048,
                priority: 1,
            }
        }
    }

    impl From<Attributes> for Parameters {
        fn from(attrs: Attributes) -> Self {
            let default = Self::default();

            let stack_size = attrs.stack_size.map_or(default.stack_size, |l| {
                parse_base10_or_panic(&l, "stack_size")
            });

            let priority = attrs
                .priority
                .map_or(default.priority, |l| parse_base10_or_panic(&l, "priority"));

            Self {
                stack_size,
                priority,
            }
        }
    }

    /// Parse a base-10 integer literal.
    ///
    /// # Panics
    ///
    /// Panics if parsing fails.
    fn parse_base10_or_panic<I>(lit_int: &syn::LitInt, attr: &str) -> I
    where
        I: core::str::FromStr,
        <I as core::str::FromStr>::Err: std::fmt::Display,
    {
        if let Ok(int) = lit_int.base10_parse() {
            assert!(
                lit_int.suffix().is_empty(),
                "`{attr}` must be a base-10 integer without a suffix",
            );
            int
        } else {
            panic!("`{attr}` must be a base-10 integer");
        }
    }

    #[derive(Default)]
    pub struct Attributes {
        pub autostart: bool,
        pub stack_size: Option<syn::LitInt>,
        pub priority: Option<syn::LitInt>,
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
