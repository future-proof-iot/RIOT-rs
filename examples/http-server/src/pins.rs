#[cfg(feature = "button-reading")]
use ariel_os::hal::peripherals;

#[cfg(all(feature = "button-reading", builder = "nrf52840dk"))]
ariel_os::hal::define_peripherals!(Button { btn1: P0_11 });

#[cfg(context = "nrf5340dk")]
ariel_os::hal::define_peripherals!(Button { btn1: P0_23 });

#[cfg(context = "st-nucleo-wb55")]
ariel_os::hal::define_peripherals!(Button { btn1: PC4 });

ariel_os::hal::group_peripherals!(Peripherals {
    #[cfg(feature = "button-reading")]
    button: Button,
});
