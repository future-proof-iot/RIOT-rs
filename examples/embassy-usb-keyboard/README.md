# embassy-usb-keyboard

## About

This application is testing basic
[embassy](https://github.com/embassy-rs/embassy) _USB HID_ usage with Ariel OS.

## How to run

In this folder, run

    laze build -b nrf52840dk run

With the device USB cable connected, pressing Button 1 should send the keycode
0x04 ('a') to the connected computer, and pressing Button 2 should send keycode
0x05 ('b').
