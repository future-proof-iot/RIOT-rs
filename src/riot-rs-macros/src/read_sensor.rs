///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro]
pub fn read_sensor(input: TokenStream) -> TokenStream {
    use syn::Ident;
    use quote::quote;

    let sensor_ident: Ident = syn::parse_macro_input!(input);

    let sensor_type_list = &["riot_rs::embassy::arch::internal_temp::InternalTemp"];

    let sensor_type_list = sensor_type_list.map(|s| s);

    let riot_rs_crate = utils::riot_rs_crate();

    let expanded = quote! {
        #riot_rs_crate::sensors::_read_sensor!(#sensor_ident, #(#sensor_type_list),*)
    };

    TokenStream::from(expanded)
}
