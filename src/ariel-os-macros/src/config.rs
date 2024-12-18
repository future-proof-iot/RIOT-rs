/// Allows to provide configuration for the associated driver during initial system configuration.
///
/// **Important**: for this configuration to be taken into account, a specific Cargo feature may
/// need to be enabled on the `ariel-os` dependency, for each configuration type (see table below).
///
/// There is no requirements on the constant's name.
///
/// # Parameters
///
/// - The name of the driver the constant provides configuration for.
///
/// | Driver    | Expected type                  | Cargo feature to enable   |
/// | --------- | ------------------------------ | ------------------------- |
/// | `network` | `embassy_net::Config`          | `override-network-config` |
/// | `usb`     | `embassy_usb::Config`          | `override-usb-config`     |
///
/// # Note
///
/// The `ariel_os` crate provides re-exports for the relevant Embassy crates.
///
/// # Examples
///
/// The following provides configuration for the network stack:
///
/// ```ignore
/// use ariel_os::reexports::embassy_net;
///
/// #[ariel_os::config(network)]
/// const NETWORK_CONFIG: embassy_net::Config = {
///     use embassy_net::Ipv4Address;
///
///     embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
///         address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
///         dns_servers: heapless::Vec::new(),
///         gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
///     })
/// };
/// ```
///
/// # Panics
///
/// This macro panics when the `ariel-os` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn config(args: TokenStream, item: TokenStream) -> TokenStream {
    #[allow(clippy::wildcard_imports)]
    use config_macro::*;

    use quote::{format_ident, quote};

    let mut attrs = ConfigAttributes::default();
    let config_attr_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with config_attr_parser);

    let config_const = syn::parse_macro_input!(item as syn::ItemConst);
    let config_const_name = &config_const.ident;

    let ariel_os_crate = utils::ariel_os_crate();

    let (config_fn_name, return_type) = match attrs.kind {
        Some(ConfigKind::Network) => (
            format_ident!("__ariel_os_network_config"),
            quote! {#ariel_os_crate::reexports::embassy_net::Config},
        ),
        Some(ConfigKind::Usb) => (
            format_ident!("__ariel_os_usb_config"),
            quote! {#ariel_os_crate::reexports::embassy_usb::Config<'static>},
        ),
        None => {
            panic!("a configuration kind must be specified");
        }
    };

    // Place the provided constant into a function whose type signature we enforce.
    // This is important as that function will be called unsafely via FFI.
    let expanded = quote! {
        #config_const

        #[no_mangle]
        fn #config_fn_name() -> #return_type {
            #config_const_name
        }
    };

    TokenStream::from(expanded)
}

mod config_macro {
    #[derive(Default)]
    pub struct ConfigAttributes {
        pub kind: Option<ConfigKind>,
    }

    impl ConfigAttributes {
        /// Parses macro attributes.
        ///
        /// # Errors
        ///
        /// Returns an error when an unsupported parameter is found.
        pub fn parse(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
            use enum_iterator::all;

            for (config_name, kind) in all::<ConfigKind>().map(|c| (c.as_name(), c)) {
                if meta.path.is_ident(config_name) {
                    self.check_only_one_kind(config_name);
                    self.kind = Some(kind);
                    return Ok(());
                }
            }

            let supported_params = all::<ConfigKind>()
                .map(|c| format!("`{}`", c.as_name()))
                .collect::<Vec<_>>()
                .join(", ");
            Err(meta.error(format!(
                "unsupported parameter ({supported_params} are supported)",
            )))
        }

        /// Checks that the macro is used for only one kind of configuration.
        ///
        /// # Panics
        ///
        /// Panics if multiple kinds are found.
        fn check_only_one_kind(&self, param: &str) {
            assert!(
                self.kind.is_none(),
                "only one configuration is supported at a time, use a separate constant for `{param}` configuration",
            );
        }
    }

    #[derive(Debug, enum_iterator::Sequence)]
    pub enum ConfigKind {
        Network,
        Usb,
    }

    impl ConfigKind {
        pub fn as_name(&self) -> &'static str {
            match self {
                Self::Network => "network",
                Self::Usb => "usb",
            }
        }
    }
}
