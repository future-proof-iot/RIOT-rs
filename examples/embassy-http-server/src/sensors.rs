use embassy_executor::Spawner;
use riot_rs::embassy::arch::peripherals;

// TODO: this whole module should get auto-generated

#[cfg(context = "nrf52")]
pub static TEMP_SENSOR: riot_rs::embassy::arch::internal_temp::InternalTemp =
    riot_rs::embassy::arch::internal_temp::InternalTemp::new();
#[cfg(context = "nrf52")]
#[riot_rs::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::linkme)]
static TEMP_SENSOR_REF: &'static dyn riot_rs::sensors::sensor::Sensor = &TEMP_SENSOR;

#[cfg(context = "nrf52")]
#[riot_rs::spawner(autostart, peripherals)]
fn temp_sensor_init(spawner: Spawner, peripherals: TempPeripherals) {
    TEMP_SENSOR.init(spawner, peripherals.p);
}

riot_rs::define_peripherals!(TempPeripherals { p: TEMP });

#[cfg(feature = "button-readings")]
pub static BUTTON_1: riot_rs::embassy::arch::PushButtonNrf =
    riot_rs::embassy::arch::PushButtonNrf::new();
#[cfg(feature = "button-readings")]
#[riot_rs::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::linkme)]
static BUTTON_1_REF: &'static dyn riot_rs::sensors::sensor::Sensor = &BUTTON_1;

#[cfg(feature = "button-readings")]
#[riot_rs::spawner(autostart, peripherals)]
fn button_1_init(_spawner: Spawner, peripherals: Button1Peripherals) {
    // FIXME: how to codegen this?
    BUTTON_1.init(embassy_nrf::gpio::Input::new(
        peripherals.p,
        embassy_nrf::gpio::Pull::Up,
    ));
}

riot_rs::define_peripherals!(Button1Peripherals { p: P0_11 });
