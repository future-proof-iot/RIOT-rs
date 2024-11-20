#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use ariel_os::usb::UsbBuilderHook;

// FAIL: using hooks require the task to be autostart
#[ariel_os::task(usb_builder_hook)]
async fn main() {}
