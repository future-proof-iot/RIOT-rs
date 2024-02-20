// TODO: document default values (which may be platform-dependent)
// TODO: document valid values
/// Runs the function decorated with this attribute macro as a separate thread.
///
/// # Parameters
///
/// - `stacksize`: (*optional*) the size of the stack allocated to the thread (in bytes)
/// - `priority`: (*optional*) the thread's priority
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
    use quote::{format_ident, quote};

    let mut attrs = ThreadAttributes::default();
    let thread_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with thread_parser);

    let thread_function = syn::parse_macro_input!(item as syn::ItemFn);

    let no_mangle_attr = if attrs.no_mangle {
        quote! {#[no_mangle]}
    } else {
        quote! {}
    };

    let fn_name = thread_function.sig.ident.clone();
    let slice_fn_name_ident = format_ident!("__start_thread_{fn_name}");
    let ThreadParameters {
        stack_size,
        priority,
    } = ThreadParameters::from(attrs);

    let riot_rs_crate = utils::riot_rs_crate();

    let expanded = quote! {
        #no_mangle_attr
        #[inline(always)]
        #thread_function

        #[#riot_rs_crate::linkme::distributed_slice(#riot_rs_crate::thread::THREAD_FNS)]
        #[linkme(crate = #riot_rs_crate::linkme)]
        fn #slice_fn_name_ident() {
            fn trampoline(_arg: ()) {
                #fn_name();
            }
            let stack = #riot_rs_crate::static_cell::make_static!([0u8; #stack_size as usize]);
            #riot_rs_crate::thread::thread_create(trampoline, (), stack, #priority);
        }
    };

    TokenStream::from(expanded)
}

struct ThreadParameters {
    stack_size: u64,
    priority: u8,
}

impl Default for ThreadParameters {
    fn default() -> Self {
        // TODO: proper values
        Self {
            stack_size: 2048,
            priority: 1,
        }
    }
}

impl From<ThreadAttributes> for ThreadParameters {
    fn from(attrs: ThreadAttributes) -> Self {
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
struct ThreadAttributes {
    stack_size: Option<syn::LitInt>,
    priority: Option<syn::LitInt>,
    no_mangle: bool,
}

impl ThreadAttributes {
    fn parse(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("stacksize") {
            self.stack_size = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("priority") {
            self.priority = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("no_mangle") {
            self.no_mangle = true;
            Ok(())
        } else {
            Err(meta.error("unsupported parameter"))
        }
    }
}
