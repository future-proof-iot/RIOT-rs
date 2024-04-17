/// Reads a sensor from a sensor trait object.
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

    let hwconfig = hwconfig::HwConfig::read_from_file().unwrap();
    dbg!(&hwconfig);

    let sensor_type_list = hwconfig.sensors().iter().map(hwconfig::Sensor::driver);
    let sensor_type_list = sensor_type_list.map(parse_type_path);

    let riot_rs_crate = utils::riot_rs_crate();

    // The `_read_sensor` macro expects a trailing comma
    let expanded = quote! {
        #riot_rs_crate::sensors::_read_sensor!(#sensor_ident, #(#sensor_type_list),* ,)
    };

    TokenStream::from(expanded)
}

// TODO: move this to a separate crate
mod hwconfig {
    use std::{collections::HashMap, fs};

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct HwConfig {
        sensors: Vec<Sensor>,
    }

    impl HwConfig {
        pub fn read_from_file() -> Result<Self, Error> {
            let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            let file_path = root.join("hw-config.yml");

            let file = fs::File::open(file_path).unwrap(); // FIXME: handle the error
            let hwconfig = serde_yaml::from_reader(&file).unwrap(); // FIXME: handle the error

            Ok(hwconfig)
        }

        pub fn sensors(&self) -> &[Sensor] {
            &self.sensors
        }
    }

    // TODO
    #[derive(Debug)]
    pub enum Error {
        ConfigNotFound,
        YamlError,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Sensor {
        name: String,
        driver: String,
        on: Option<String>,
        when: Option<String>,
        peripherals: Option<Peripherals>,
    }

    impl Sensor {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn driver(&self) -> &str {
            &self.driver
        }

        pub fn on(&self) -> Option<&str> {
            self.on.as_deref()
        }

        pub fn when(&self) -> Option<&str> {
            self.when.as_deref()
        }

        pub fn peripherals(&self) -> Option<&Peripherals> {
            self.peripherals.as_ref()
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct Peripherals(HashMap<String, String>);

    impl Peripherals {
        pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
            self.0.iter()
        }
    }
}

// TODO: is there a ready-made version of this function in the syn crate?
fn parse_type_path(type_path: &str) -> proc_macro2::TokenStream {
    let path_segments = type_path
        .split("::")
        .map(|seg| quote::format_ident!("{seg}"));

    quote::quote! {#(#path_segments)::*}
}
