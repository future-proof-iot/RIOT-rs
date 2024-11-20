use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, peripherals, rng};

#[cfg(not(any(feature = "stm32-hash-rng", feature = "stm32-rng")))]
compile_error!("no stm32 rng feature enabled");

#[cfg(feature = "stm32-hash-rng")]
bind_interrupts!(struct Irqs {
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[cfg(feature = "stm32-rng")]
bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

pub fn construct_rng(peripherals: &mut crate::OptionalPeripherals) {
    cfg_if::cfg_if! {
        // The union of all contexts that wind up in a construct_rng should be synchronized
        // with laze-project.yml's hwrng module.
        if #[cfg(any(context = "stm32"))] {
            let rng = Rng::new(
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
