use quote::format_ident;

const RIOT_RS_CRATE_NAME: &str = "riot-rs";

/// Returns a [`struct@syn::Ident`] identifying the `riot-rs` dependency.
///
/// # Panics
///
/// - Panics when the `riot-rs` crate cannot be found as a dependency of the crate in which
/// this function is called.
/// - Panics if `riot-rs` is used as a dependency of itself.
pub fn riot_rs_crate() -> syn::Ident {
    let riot_rs_crate = proc_macro_crate::crate_name(RIOT_RS_CRATE_NAME)
        .unwrap_or_else(|_| panic!("{RIOT_RS_CRATE_NAME} should be present in `Cargo.toml`"));

    match riot_rs_crate {
        proc_macro_crate::FoundCrate::Itself => {
            panic!(
                "{} cannot be used as a dependency of itself",
                env!("CARGO_CRATE_NAME"),
            );
        }
        proc_macro_crate::FoundCrate::Name(riot_rs_crate) => format_ident!("{riot_rs_crate}"),
    }
}

// TODO: is there a ready-made version of this function in the syn crate?
pub fn parse_type_path(type_path: &str) -> proc_macro2::TokenStream {
    let path_segments = type_path
        .split("::")
        .map(|seg| quote::format_ident!("{seg}"));

    quote::quote! {#(#path_segments)::*}
}
