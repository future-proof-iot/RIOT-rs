//! Use and create executor ISR from string name.
//!
//! This module provides [`executor_swi`], a macro that generates the interrupt
//! that polls the executor.
//!

/// Create swi interrupt routine and import from string interrupt name.
///
/// The macro turns this:
///
/// ```Rust
/// executor_swi!(SWI_IRQ_1);
/// ```
///
/// into this:
///
/// ```Rust
/// pub use interrupt::SWI_IRQ_1 as SWI;
/// #[interrupt]
/// unsafe fn SWI_IRQ_1() {
///     unsafe { crate::EXECUTOR.on_interrupt() }
/// }
/// ```
///
/// Note: this expects the `interrupt` to be present (e.g., "used") and that it contains the ISR
/// type.
#[macro_export]
macro_rules! executor_swi {
    ($swi:ident) => {
        pub use interrupt::$swi as SWI;
        #[interrupt]
        unsafe fn $swi() {
            // SAFETY:
            // - As required, it is called from an ISR
            // - The interrupt is enabled by start(), thus this is not called before start.
            //   (This macro just adds "only enable it after starting the executor" to the
            //   requirements of the unsafe interrupt starting; the safe start() function
            //    trusts the user to pass the right number.)
            unsafe { crate::EXECUTOR.on_interrupt() }
        }
    };
}
