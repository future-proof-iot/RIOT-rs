#![no_main]
#![no_std]
#![feature(raw_ref_macros)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(const_maybe_uninit_as_ptr)]

extern crate cortex_m;
use clist::{Link, TypedList};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::Peripherals;
//use cortex_m::peripheral::SCB;

use riot_rs_rt::debug::println;

#[derive(Clone, Copy)]
struct TestStruct {
    value: usize,
    next: Link,
}

const OFF: usize = clist::offset_of!(TestStruct, next);
const N: usize = 128;
pub fn clist_bench() -> usize {
    let mut links: [TestStruct; N] = [TestStruct {
        next: Link::new(),
        value: 0,
    }; N];

    let mut list: TypedList<TestStruct, OFF> = TypedList::new();
    let mut i = 0;
    for link in &mut links {
        link.value = i;
        list.lpush(link);
        i += 1;
    }
    let mut sum = 0;
    for _ in 0..N {
        let elem = list.lpop().unwrap();
        sum += elem.value;
    }

    return sum;
}

#[no_mangle]
fn user_main() {
    let mut p = Peripherals::take().unwrap();
    //
    p.SCB.clear_sleepdeep();

    //
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(0x00FFFFFF);
    p.SYST.clear_current();
    p.SYST.enable_counter();

    const N: usize = 100;

    while cortex_m::peripheral::SYST::get_current() == 0 {}

    let before = cortex_m::peripheral::SYST::get_current();

    let mut sum: usize = 0;
    for _ in 0..N {
        sum += clist_bench();
    }

    let total = before - cortex_m::peripheral::SYST::get_current();

    assert!(!p.SYST.has_wrapped());

    println!("{}", sum);
    println!("total: {} ticks: {}", total, total as usize / N);
}
