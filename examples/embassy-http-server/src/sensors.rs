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

#[riot_rs::spawner(autostart, peripherals)]
fn sensor_init(spawner: Spawner, peripherals: SensorPeripherals) {
    #[cfg(context = "nrf52840")]
    {
        use riot_rs::sensors::{
            sensor::{PhysicalValue, ThresholdKind},
            Sensor,
        };

        TEMP_SENSOR.init(spawner, peripherals.temp);

        let threshold = PhysicalValue::new(2300);
        TEMP_SENSOR.set_threshold(ThresholdKind::Lower, threshold);
        TEMP_SENSOR.set_threshold_enabled(ThresholdKind::Lower, true);
    }
}

riot_rs::define_peripherals!(SensorPeripherals { temp: TEMP });
