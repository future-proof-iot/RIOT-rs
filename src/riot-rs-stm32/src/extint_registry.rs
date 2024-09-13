use embassy_stm32::{
    exti::{AnyChannel, Channel},
    gpio::Pin,
    peripherals, OptionalPeripherals, Peripheral,
};
use portable_atomic::{AtomicBool, AtomicU16, Ordering};
use riot_rs_embassy_common::gpio::input::InterruptError;

pub static EXTINT_REGISTRY: ExtIntRegistry = ExtIntRegistry::new();

pub struct ExtIntRegistry {
    initialized: AtomicBool,
    used_interrupt_channels: AtomicU16, // 16 channels
}

impl ExtIntRegistry {
    // Collect all channel peripherals so that the registry is the only one managing them.
    const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            used_interrupt_channels: AtomicU16::new(0),
        }
    }

    pub fn init(&self, peripherals: &mut OptionalPeripherals) {
        peripherals.EXTI0.take().unwrap();
        peripherals.EXTI1.take().unwrap();
        peripherals.EXTI2.take().unwrap();
        peripherals.EXTI3.take().unwrap();
        peripherals.EXTI4.take().unwrap();
        peripherals.EXTI5.take().unwrap();
        peripherals.EXTI6.take().unwrap();
        peripherals.EXTI7.take().unwrap();
        peripherals.EXTI8.take().unwrap();
        peripherals.EXTI9.take().unwrap();
        peripherals.EXTI10.take().unwrap();
        peripherals.EXTI11.take().unwrap();
        peripherals.EXTI12.take().unwrap();
        peripherals.EXTI13.take().unwrap();
        peripherals.EXTI14.take().unwrap();
        peripherals.EXTI15.take().unwrap();

        self.initialized.store(true, Ordering::Release);

        // Do nothing else, just consume the peripherals: they are ours now!
    }

    pub fn get_interrupt_channel_for_pin<P: Peripheral<P = T>, T: Pin>(
        &self,
        pin: P,
    ) -> Result<AnyChannel, InterruptError> {
        // Make sure that the interrupt channels have been captured during initialization.
        assert!(self.initialized.load(Ordering::Acquire));

        let pin = pin.into_ref().map_into();
        let pin_number = pin.pin();

        // As interrupt channels are mutually exclusive between ports (ie., if channel i has
        // been bound for pin i of a port, it cannot be used for pin i of another port), we
        // only check the pin number.
        // NOTE(ordering): since setting a bit is an idempotent operation, and since we do not
        // allow clearing them, the ordering does not matter.
        let was_used = self
            .used_interrupt_channels
            .bit_set(pin_number.into(), Ordering::Relaxed);

        if was_used {
            return Err(InterruptError::IntChannelAlreadyUsed);
        }

        // They are the same
        let ch_number = pin_number;

        // NOTE(embassy): ideally we would be using `T::ExtiChannel::steal()` instead of this
        // match, but Embassy does not provide this.
        // SAFETY: this function enforces that the same channel cannot be obtained twice,
        // making sure multiple instances are not used at the same time as the mandatory
        // `init()` method has collected all channel peripherals beforehand.
        let ch = match ch_number {
            0 => unsafe { peripherals::EXTI0::steal() }.degrade(),
            1 => unsafe { peripherals::EXTI1::steal() }.degrade(),
            2 => unsafe { peripherals::EXTI2::steal() }.degrade(),
            3 => unsafe { peripherals::EXTI3::steal() }.degrade(),
            4 => unsafe { peripherals::EXTI4::steal() }.degrade(),
            5 => unsafe { peripherals::EXTI5::steal() }.degrade(),
            6 => unsafe { peripherals::EXTI6::steal() }.degrade(),
            7 => unsafe { peripherals::EXTI7::steal() }.degrade(),
            8 => unsafe { peripherals::EXTI8::steal() }.degrade(),
            9 => unsafe { peripherals::EXTI9::steal() }.degrade(),
            10 => unsafe { peripherals::EXTI10::steal() }.degrade(),
            11 => unsafe { peripherals::EXTI11::steal() }.degrade(),
            12 => unsafe { peripherals::EXTI12::steal() }.degrade(),
            13 => unsafe { peripherals::EXTI13::steal() }.degrade(),
            14 => unsafe { peripherals::EXTI14::steal() }.degrade(),
            15 => unsafe { peripherals::EXTI15::steal() }.degrade(),
            _ => unreachable!(),
        };

        Ok(ch)
    }
}
