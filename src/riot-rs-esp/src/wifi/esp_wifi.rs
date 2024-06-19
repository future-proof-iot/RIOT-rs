use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitialization,
};
use once_cell::sync::OnceCell;
use riot_rs_debug::log::{debug, info};

pub type NetworkDevice = WifiDevice<'static, WifiStaDevice>;

// Ideally, all Wi-Fi initialization would happen here.
// Unfortunately that's complicated, so we're using WIFI_INIT to pass the
// `EspWifiInitialization` from `crate::arch::esp::init()`.
// Using a `once_cell::OnceCell` here for critical-section support, just to be
// sure.
pub static WIFI_INIT: OnceCell<EspWifiInitialization> = OnceCell::new();

#[cfg(feature = "threading")]
pub static WIFI_THREAD_ID: OnceCell<riot_rs_threads::ThreadId> = OnceCell::new();

pub fn init(peripherals: &mut crate::OptionalPeripherals, spawner: Spawner) -> NetworkDevice {
    let wifi = peripherals.WIFI.take().unwrap();
    let init = WIFI_INIT.get().unwrap();
    let (device, controller) = esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();

    spawner.spawn(connection(controller)).ok();

    device
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    #[cfg(feature = "threading")]
    {
        let thread_id = WIFI_THREAD_ID.get().unwrap();

        // Disable esp-wifi interrupts that are initialized in esp-wifi
        // until the `esp_wifi_thread` runs.
        interrupt::disable(esp_hal::Cpu::ProCpu, Interrupt::FROM_CPU_INTR3);
        interrupt::disable(esp_hal::Cpu::ProCpu, Interrupt::SYSTIMER_TARGET0);
        // Wake-up the `esp_wifi_thread`.
        riot_rs_threads::thread_flags::set(*thread_id, 0b1);
    }

    debug!("start connection task");

    #[cfg(not(feature = "defmt"))]
    debug!("Device capabilities: {:?}", controller.get_capabilities());

    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_secs(5)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            debug!("Configuring Wi-Fi");
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: crate::wifi::WIFI_NETWORK.try_into().unwrap(),
                password: crate::wifi::WIFI_PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            debug!("Starting Wi-Fi");
            controller.start().await.unwrap();
            debug!("Wi-Fi started!");
        }
        debug!("About to connect...");

        match controller.connect().await {
            Ok(_) => info!("Wifi connected!"),
            Err(e) => {
                info!("Failed to connect to Wi-Fi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[cfg(feature = "threading")]
mod wifi_thread {
    use esp_hal::{
        interrupt,
        peripherals::{Interrupt, SYSTIMER},
    };

    #[cfg(context = "esp32c6")]
    use esp_hal::peripherals::INTPRI as SystemPeripheral;
    #[cfg(context = "esp32c3")]
    use esp_hal::peripherals::SYSTEM as SystemPeripheral;

    use super::*;

    // Handle the systimer alarm 0 interrupt, configured in esp-wifi.
    extern "C" fn systimer_target0_() {
        // SAFETY: constant pointer to register block is valid.
        let systimer = unsafe { &*SYSTIMER::PTR };
        // Clear interrupt.
        systimer.int_clr().write(|w| w.target0().clear_bit_by_one());
        // Wake up `esp_wifi_thread`.
        if !riot_rs_threads::wakeup(*WIFI_THREAD_ID.get().unwrap()) {
            // We're already in the context of `esp_wifi_thread`, so yield
            // directly.
            yield_to_esp_wifi_scheduler();
        }
    }

    fn yield_to_esp_wifi_scheduler() {
        // SAFETY: constant pointer to register block is valid.
        let ptr = unsafe { &*SystemPeripheral::PTR };
        // CPU Interrupt 3 triggers the scheduler in `esp-wifi`.
        ptr.cpu_intr_from_cpu_3()
            .modify(|_, w| w.cpu_intr_from_cpu_3().set_bit());
    }

    // Thread that runs the esp-wifi scheduler.
    ///
    /// Because it runs at highest priority, it can't be preempted by any riot-rs threads and therefore
    /// the two schedulers won't interleave.
    #[riot_rs_macros::thread(autostart, priority = riot_rs_threads::SCHED_PRIO_LEVELS as u8 - 1)]
    fn esp_wifi_thread() {
        WIFI_THREAD_ID
            .set(riot_rs_threads::current_pid().unwrap())
            .unwrap();

        // Wait until `embassy` is initialized.
        riot_rs_threads::thread_flags::wait_one(0b1);

        // Bind the periodic systimer that is configured in esp-wifi to our own handler.
        //
        // SAFETY: This overwrites the existing handler from esp-wifi, which is okay because
        // we want to handle the interrupt differently. It needs to be done after the esp-hal
        // initialization finished.
        unsafe {
            interrupt::bind_interrupt(
                Interrupt::SYSTIMER_TARGET0,
                core::mem::transmute(systimer_target0_ as *const ()),
            );
        }
        interrupt::enable(Interrupt::SYSTIMER_TARGET0, interrupt::Priority::Priority2).unwrap();

        loop {
            interrupt::enable(Interrupt::FROM_CPU_INTR3, interrupt::Priority::Priority1).unwrap();
            // Yield to the esp-wifi scheduler tasks, so that they get a chance to run.
            yield_to_esp_wifi_scheduler();
            // Disable esp-wifi scheduler so that it won't interleave with the riot-rs-threads scheduler.
            interrupt::disable(esp_hal::Cpu::ProCpu, Interrupt::FROM_CPU_INTR3);
            // Sleep until the systimer alarm 0 interrupts again.
            riot_rs_threads::sleep()
        }
    }
}
