# usb-keyboard

## About

This application is testing basic
[embassy](https://github.com/embassy-rs/embassy) _USB HID_ usage with Ariel OS.

## How to run

In this folder, run

    laze build -b nrf52840dk run

With the device USB cable connected, pressing a button will send its associated
keycode to the attached computer, based on the following table:

| Button   | Keycode    |
| -------- | ---------- |
| Button 1 | 'a' (0x04) |
| Button 2 | 'c' (0x06) |
| Button 3 | 'g' (0x0a) |
| Button 4 | 't' (0x17) |
