use crate::{arch, DefinePeripheralsError, Drivers, InitializationArgs};

/// Defines an application.
///
/// Allows to separate its fallible initialization from its infallible running phase.
pub trait Application {
    /// Applications must implement this to obtain the peripherals they require.
    ///
    /// This function is only run once at startup and instantiates the application.
    /// No guarantee is provided regarding the order in which different applications are
    /// initialized.
    /// The [`assign_resources!`] macro can be leveraged to extract the required peripherals.
    fn initialize(
        peripherals: &mut arch::OptionalPeripherals,
        init_args: InitializationArgs,
    ) -> Result<&dyn Application, ApplicationInitError>
    where
        Self: Sized;

    /// After an application has been initialized, this method is called by the system to start the
    /// application.
    ///
    /// This function must not block but may spawn [Embassy tasks](embassy_executor::task) using
    /// the provided [`Spawner`](embassy_executor::Spawner).
    /// In addition, it is provided with the drivers initialized by the system.
    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers);
}

/// Represents errors that can happen during application initialization.
#[derive(Debug)]
pub enum ApplicationInitError {
    /// The application could not obtain a peripheral, most likely because it was already used by
    /// another application or by the system itself.
    CannotTakePeripheral,
}

impl From<DefinePeripheralsError> for ApplicationInitError {
    fn from(err: DefinePeripheralsError) -> Self {
        match err {
            DefinePeripheralsError::TakingPeripheral => Self::CannotTakePeripheral,
        }
    }
}

/// Sets the [`Application::initialize()`] function implemented on the provided type to be run at
/// startup.
#[macro_export]
macro_rules! riot_initialize {
    ($prog_type:ident) => {
        #[$crate::distributed_slice($crate::EMBASSY_TASKS)]
        #[linkme(crate = $crate::linkme)]
        fn __init_application(
            peripherals: &mut $crate::arch::OptionalPeripherals,
            init_args: $crate::InitializationArgs,
        ) -> Result<&dyn $crate::Application, $crate::ApplicationInitError> {
            <$prog_type as Application>::initialize(peripherals, init_args)
        }
    };
}
