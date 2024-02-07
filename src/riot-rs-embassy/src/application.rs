use crate::{arch, DefinePeripheralsError, Drivers};

#[cfg(feature = "usb")]
use crate::UsbBuilder;

/// Defines an application.
///
/// Allows to separate its fallible initialization from its infallible running phase.
pub trait Application {
    /// Creates the trait object so that we can then call methods on it.
    ///
    /// # Note
    ///
    /// This function must be callable multiple times.
    fn init() -> &'static dyn Application
    where
        Self: Sized;

    /// Allows to mutate the [USB builder](UsbBuilder) to configure USB on the device.
    #[cfg(feature = "usb")]
    fn usb_builder_hook(&self, _usb_builder: &mut UsbBuilder) -> Result<(), UsbBuilderHookError> {
        Ok(())
    }

    /// This method is run once at startup and is intended to start the application.
    /// It must not block but may spawn [Embassy tasks](embassy_executor::task) using the provided
    /// [`Spawner`](embassy_executor::Spawner).
    /// The [`define_peripherals!`](crate::define_peripherals!) macro can be leveraged to extract
    /// the required peripherals.
    /// In addition, this method is provided with the drivers initialized by the system, which
    /// can be configured using the hook methods on this trait.
    ///
    /// # Note
    ///
    /// No guarantee is provided regarding the order in which different applications are started.
    fn start(
        &self,
        peripherals: &mut arch::OptionalPeripherals,
        spawner: embassy_executor::Spawner,
        drivers: Drivers,
    ) -> Result<(), ApplicationError>;
}

/// Represents errors that can happen during application initialization.
#[derive(Debug)]
pub enum ApplicationError {
    /// The application could not obtain a peripheral, most likely because it was already used by
    /// another application or by the system itself.
    CannotTakePeripheral,
}

impl From<DefinePeripheralsError> for ApplicationError {
    fn from(err: DefinePeripheralsError) -> Self {
        match err {
            DefinePeripheralsError::TakingPeripheral => Self::CannotTakePeripheral,
        }
    }
}

/// Represents an error happening within the USB builder hook.
#[cfg(feature = "usb")]
#[derive(Debug)]
pub struct UsbBuilderHookError;

/// Sets the [`Application::init()`] function implemented on the provided type to be run at
/// startup.
#[macro_export]
macro_rules! riot_initialize {
    ($prog_type:ident) => {
        #[$crate::distributed_slice($crate::EMBASSY_TASKS)]
        #[linkme(crate = $crate::linkme)]
        fn __init_application() -> &'static dyn $crate::Application {
            <$prog_type as Application>::init()
        }
    };
}
