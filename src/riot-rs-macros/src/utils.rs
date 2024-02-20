use quote::format_ident;

const RIOT_RS_CRATE_NAME: &str = "riot-rs";

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
        proc_macro_crate::FoundCrate::Name(riot_rs_crate) => format_ident!("{}", riot_rs_crate),
    }
}
