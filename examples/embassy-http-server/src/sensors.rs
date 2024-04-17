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
fn sensor_init(spawner: Spawner, peripherals: TempPeripherals) {
    TEMP_SENSOR.init(spawner, peripherals.p);
}

riot_rs::define_peripherals!(TempPeripherals { p: TEMP });
