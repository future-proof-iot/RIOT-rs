///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro]
pub fn read_sensor(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::Ident;

    let sensor_ident: Ident = syn::parse_macro_input!(input);

    let sensor_type_list = &["riot_rs::embassy::arch::internal_temp::InternalTemp"];

    let sensor_type_list = sensor_type_list.map(parse_type_path);

    let riot_rs_crate = utils::riot_rs_crate();

    // The `_read_sensor` macro expects a trailing comma
    let expanded = quote! {
        #riot_rs_crate::sensors::_read_sensor!(#sensor_ident, #(#sensor_type_list),* ,)
    };

    TokenStream::from(expanded)
}

// TODO: is there a ready-made version of this function in the syn crate?
fn parse_type_path(type_path: &str) -> proc_macro2::TokenStream {
    let path_segments = type_path
        .split("::")
        .map(|seg| quote::format_ident!("{seg}"));

    quote::quote! {#(#path_segments)::*}
}
