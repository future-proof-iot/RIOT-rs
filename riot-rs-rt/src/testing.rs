// #![feature(custom_test_frameworks)]
// #![test_runner(riot_rs_rt::testing::test_runner)]
// #![reexport_test_harness_main = "test_main"]

use super::debug;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        debug::print!("{}...\t", core::any::type_name::<T>());
        self();
        debug::println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    debug::println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    debug::println!("Done.");
    debug::exit(debug::EXIT_SUCCESS);
}
