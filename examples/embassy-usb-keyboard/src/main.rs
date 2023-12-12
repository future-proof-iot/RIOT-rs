#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::embassy::{usbd_hid::descriptor::KeyboardReport, TaskArgs};
use riot_rs::rt::debug::println;

use embassy_nrf::gpio::{Input, Pin, Pull};

#[embassy_executor::task]
async fn set_up_keyboard(args: TaskArgs) {
    let mut button = Input::new(args.P0_11.take().unwrap().degrade(), Pull::Up);

    loop {
        button.wait_for_low().await;
        println!("PRESSED");

        let report = KeyboardReport {
            keycodes: [4, 0, 0, 0, 0, 0],
            leds: 0,
            modifier: 0,
            reserved: 0,
        };

        if let Err(e) = args.hid_writer.lock().await.write_serialize(&report).await {
            println!("Failed to send report: {:?}", e);
        }

        button.wait_for_high().await;
        println!("RELEASED");
        let report = KeyboardReport {
            keycodes: [0, 0, 0, 0, 0, 0],
            leds: 0,
            modifier: 0,
            reserved: 0,
        };
        match args.hid_writer.lock().await.write_serialize(&report).await {
            Ok(()) => {}
            Err(e) => println!("Failed to send report: {:?}", e),
        };
    }
}

#[linkme::distributed_slice(riot_rs::embassy::EMBASSY_TASKS)]
fn __start_usb_keyboard(spawner: embassy_executor::Spawner, t: TaskArgs) {
    spawner.spawn(set_up_keyboard(t)).unwrap();
}

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );

    loop {}
}
