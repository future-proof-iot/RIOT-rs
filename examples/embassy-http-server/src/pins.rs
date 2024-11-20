#[cfg(feature = "button-readings")]
use ariel_os::hal::peripherals;

#[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
ariel_os::define_peripherals!(Buttons {
    btn1: P0_11,
    btn2: P0_12,
    btn3: P0_24,
    btn4: P0_25,
});

ariel_os::group_peripherals!(Peripherals {
    #[cfg(feature = "button-readings")]
    buttons: Buttons,
});
