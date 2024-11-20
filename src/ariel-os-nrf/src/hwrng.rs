embassy_nrf::bind_interrupts!(struct Irqs {
    RNG => embassy_nrf::rng::InterruptHandler<embassy_nrf::peripherals::RNG>;
});

pub fn construct_rng(peripherals: &mut crate::OptionalPeripherals) {
    cfg_if::cfg_if! {
        // The union of all contexts that wind up in a construct_rng should be synchronized
        // with laze-project.yml's hwrng module.
        if #[cfg(any(context = "nrf51", context = "nrf52"))] {
            let rng = embassy_nrf::rng::Rng::new(
                peripherals
                    .RNG
                    // We don't even have to take it out, just use it to seed the RNG
                    .as_mut()
                    .expect("RNG has not been previously used"),
                Irqs,
            );

            riot_rs_random::construct_rng(rng);
        } else if #[cfg(context = "ariel-os")] {
            compile_error!("hardware RNG is not supported on this MCU family");
        }
    }
}
