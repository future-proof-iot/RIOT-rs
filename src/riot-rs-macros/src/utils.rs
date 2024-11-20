use quote::format_ident;

const RIOT_RS_CRATE_NAME: &str = "ariel-os";

/// Returns a [`struct@syn::Ident`] identifying the `ariel-os` dependency.
///
/// # Panics
///
/// - Panics when the `ariel-os` crate cannot be found as a dependency of the crate in which
///   this function is called.
/// - Panics if `ariel-os` is used as a dependency of itself.
pub fn riot_rs_crate() -> syn::Ident {
    find_crate(RIOT_RS_CRATE_NAME)
        .unwrap_or_else(|| panic!("{RIOT_RS_CRATE_NAME} should be present in `Cargo.toml`"))
}

/// Returns a [`struct@syn::Ident`] identifying the `name` dependency (or `None`).
///
/// # Panics
///
/// - Panics if `name` is used as a dependency of itself.
pub fn find_crate(name: &str) -> Option<syn::Ident> {
    if let Ok(crate_) = proc_macro_crate::crate_name(name) {
        match crate_ {
            proc_macro_crate::FoundCrate::Itself => {
                panic!("{name} cannot be used as a dependency of itself");
            }
            proc_macro_crate::FoundCrate::Name(crate_) => Some(format_ident!("{crate_}")),
        }
    } else {
        None
    }
}
