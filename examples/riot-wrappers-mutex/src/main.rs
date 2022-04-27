// Copyright (C) 2022 Kaspar Schleiser <kaspar@schleiser.de>
//
// This file is subject to the terms and conditions of the GNU Lesser
// General Public License v2.1. See the file LICENSE in the top level
// directory for more details.
#![no_main]
#![no_std]

use riot_rs as _;

use riot_wrappers::mutex::Mutex;

#[no_mangle]
fn riot_main() {
    let val = 0u32;
    let mut mutex = Mutex::new(val);
    mutex.lock();
}
