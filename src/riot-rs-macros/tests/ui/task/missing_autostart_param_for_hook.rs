#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use riot_rs::embassy::usb::UsbBuilderHook;

// FAIL: using hooks require the task to be autostart
#[riot_rs::task(usb_builder_hook)]
async fn main() {}
