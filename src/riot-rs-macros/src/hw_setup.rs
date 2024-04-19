#[proc_macro_attribute]
pub fn hw_setup(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // FIXME: check that the item is indeed just a module declaration, and reuse its name
    let mod_name = format_ident!("sensors");

    let riot_rs_crate = utils::riot_rs_crate();

    let hwsetup = HwSetup::read_from_file().unwrap();
    dbg!(&hwsetup);

    let sensors = hwsetup
        .sensors()
        .iter()
        .map(|sensor| hw_setup::generate_sensor(&riot_rs_crate, sensor));

    let expanded = quote! {
        mod #mod_name {
            use embassy_executor::Spawner;
            use #riot_rs_crate::embassy::arch::peripherals;

            #(#sensors)*
        }
    };

    TokenStream::from(expanded)
}

mod hw_setup {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::Sensor;

    pub fn generate_sensor(riot_rs_crate: &syn::Ident, sensor_setup: &Sensor) -> TokenStream {
        let sensor_name = sensor_setup.name();

        let sensor_name_static = format_ident!("{sensor_name}");
        let sensor_ref = format_ident!("{sensor_name}_REF");
        let sensor_type = crate::utils::parse_type_path(sensor_setup.driver());

        let spawner_fn = format_ident!("{sensor_name}_init");

        let peripheral_struct = format_ident!("{sensor_name}Peripherals");

        let on_conds = parse_conditional_list("context", sensor_setup.on());
        let when_conds = parse_conditional_list("feature", sensor_setup.when());

        // We have to collect the iterator because `cfg_conds` is used multiple times when
        // expanding
        let cfg_conds = on_conds.iter().chain(when_conds.iter()).collect::<Vec<_>>();
        dbg!(&cfg_conds);

        // FIXME: codegen the sensor init

        let expanded = quote! {
            // FIXME: does this work with zero cfg_conds?
            #[cfg(all(#(#cfg_conds),*))]
            pub static #sensor_name_static: #sensor_type = #sensor_type::new();

            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::linkme::distributed_slice(#riot_rs_crate::sensors::SENSOR_REFS)]
            #[linkme(crate = #riot_rs_crate::linkme)]
            static #sensor_ref: &'static dyn #riot_rs_crate::sensors::Sensor = &#sensor_name_static;

            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::spawner(autostart, peripherals)]
            fn #spawner_fn(_spawner: Spawner, peripherals: #peripheral_struct) {
            //     // FIXME: how to codegen this?
            //     BUTTON_1.init(#riot_rs_crate::embassy::arch::gpio::Input::new(
            //         peripherals.p,
            //         #riot_rs_crate::embassy::arch::gpio::Pull::Up,
            //     ));
            }

            #[cfg(all(#(#cfg_conds),*))]
            #riot_rs_crate::define_peripherals!(#peripheral_struct { });
            // FIXME
            // #riot_rs_crate::define_peripherals!(Button1Peripherals { p: P0_11 });
        };

        TokenStream::from(expanded)
    }

    fn parse_conditional_list(cfg_attr: &str, conditionals: Option<&str>) -> Vec<TokenStream> {
        if let Some(on) = conditionals {
            let context_attr = format_ident!("{cfg_attr}");

            on.split(',')
                .map(str::trim)
                .map(|context| quote!(#context_attr = #context))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }
}
