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
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::Sensor;
    use proc_macro2::TokenStream;

    pub fn generate_sensor(riot_rs_crate: &syn::Ident, sensor_setup: &Sensor) -> TokenStream {
        let sensor_name = format_ident!("{}", sensor_setup.name());
        let sensor_ref = format_ident!("{}_REF", sensor_setup.name());
        let sensor_type = crate::utils::parse_type_path(sensor_setup.driver());

        let spawner_fn = format_ident!("{}_init", sensor_setup.name());

        let peripheral_struct = format_ident!("{}Peripherals", sensor_setup.name());

        // FIXME: impl `on` (context conditional)
        // FIXME: impl `when` (feature conditional)
        // FIXME: codegen the sensor init

        let expanded = quote! {
            // #[cfg(feature = "button-readings")]
            // {
                pub static #sensor_name: #sensor_type = #sensor_type::new();

                #[#riot_rs_crate::linkme::distributed_slice(#riot_rs_crate::sensors::SENSOR_REFS)]
                #[linkme(crate = #riot_rs_crate::linkme)]
                static #sensor_ref: &'static dyn #riot_rs_crate::sensors::Sensor = &#sensor_name;

                #[#riot_rs_crate::spawner(autostart, peripherals)]
                fn #spawner_fn(_spawner: Spawner, peripherals: #peripheral_struct) {
                //     // FIXME: how to codegen this?
                //     BUTTON_1.init(#riot_rs_crate::embassy::arch::gpio::Input::new(
                //         peripherals.p,
                //         #riot_rs_crate::embassy::arch::gpio::Pull::Up,
                //     ));
                }

                #riot_rs_crate::define_peripherals!(#peripheral_struct { });
                // #riot_rs_crate::define_peripherals!(Button1Peripherals { p: P0_11 });
            // }
        };

        TokenStream::from(expanded)
    }
}
