/// Generates sensor-related enums whose number of variants needs to be adjusted based on Cargo
/// features, to accommodate the sensor driver returning the largest number of values.
///
/// One single type must be defined so that it can be used in the Future returned by sensor
/// drivers, which must be the same for every sensor driver so it can be part of the `Sensor`
/// trait.
#[proc_macro]
pub fn define_count_adjusted_sensor_enums(_item: TokenStream) -> TokenStream {
    use quote::quote;

    #[allow(clippy::wildcard_imports)]
    use define_count_adjusted_enum::*;

    // The order of these feature-gated statements is important as these features are not meant to
    // be mutually exclusive.
    #[allow(unused_variables, reason = "overridden by feature selection")]
    let count = 1;
    #[cfg(feature = "max-reading-value-min-count-2")]
    let count = 2;
    #[cfg(feature = "max-reading-value-min-count-3")]
    let count = 3;
    #[cfg(feature = "max-reading-value-min-count-4")]
    let count = 4;
    #[cfg(feature = "max-reading-value-min-count-6")]
    let count = 6;
    #[cfg(feature = "max-reading-value-min-count-7")]
    let count = 7;
    #[cfg(feature = "max-reading-value-min-count-9")]
    let count = 9;
    #[cfg(feature = "max-reading-value-min-count-12")]
    let count = 12;

    let physical_values_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([Value; #i]) }
    });
    let physical_values_first_value = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! {
            Self::#variant(values) => {
                if let Some(value) = values.first() {
                    *value
                } else {
                    // NOTE(no-panic): there is always at least one value
                    unreachable!();
                }
            }
        }
    });

    let reading_axes_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([ReadingAxis; #i]) }
    });

    let values_iter = (1..=count)
        .map(|i| {
            let variant = variant_name(i);
            quote! { Self::#variant(values) => values.iter().copied() }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        /// Values returned by a sensor driver.
        ///
        /// This type implements [`Reading`] to iterate over the values.
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is automatically adjusted.
        #[derive(Debug, Copy, Clone)]
        pub enum Values {
            #[doc(hidden)]
            #(#physical_values_variants),*
        }

        impl Reading for Values {
            fn value(&self) -> Value {
                match self {
                    #(#physical_values_first_value),*
                }
            }

            fn values(&self) -> impl ExactSizeIterator<Item = Value> {
                match self {
                    #(#values_iter),*
                }
            }
        }

        /// Metadata required to interpret values returned by [`Sensor::wait_for_reading()`].
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is automatically adjusted.
        #[derive(Debug, Copy, Clone)]
        pub enum ReadingAxes {
            #[doc(hidden)]
            #(#reading_axes_variants),*,
        }

        impl ReadingAxes {
            /// Returns an iterator over the underlying [`ReadingAxis`] items.
            ///
            /// For a given sensor driver, the number and order of items match the one of
            /// [`Values`].
            /// [`Iterator::zip()`] can be useful to zip the returned iterator with the one
            /// obtained with [`Reading::values()`].
            pub fn iter(&self) -> impl Iterator<Item = ReadingAxis> + '_ {
                match self {
                    #(#values_iter),*,
                }
            }

            /// Returns the first [`ReadingAxis`].
            pub fn first(&self) -> ReadingAxis {
                if let Some(value) = self.iter().next() {
                    value
                } else {
                    // NOTE(no-panic): there is always at least one value.
                    unreachable!();
                }
            }
        }
    };

    TokenStream::from(expanded)
}

mod define_count_adjusted_enum {
    pub fn variant_name(index: usize) -> syn::Ident {
        quote::format_ident!("V{index}")
    }
}
