#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::debug::println;

#[cfg(test)]
#[embedded_test::tests(executor = riot_rs::embassy::thread_executor::Executor::new())]
mod tests {
    // Optional: A init function which is called before every test
    #[init]
    fn init() {}

    // A test which takes the state returned by the init function (optional)
    #[test]
    async fn trivial_async() {
        assert!(true)
    }

    // A test which takes the state returned by the init function (optional)
    #[test]
    fn trivial() {
        assert!(true)
    }
}
