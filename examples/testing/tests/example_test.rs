#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

#[cfg(test)]
#[embedded_test::tests]
mod tests {
    // Optional: An init function which is called before every test
    #[init]
    fn init() -> u32 {
        return 42;
    }

    // A test which takes the state returned by the init function (optional)
    // This is an async function, it will be executed on the system executor.
    #[test]
    async fn trivial_async(n: u32) {
        assert!(n == 42)
    }

    // A test which takes the state returned by the init function (optional)
    #[test]
    fn trivial(n: u32) {
        assert!(n == 42)
    }

    // A test which is "ignored".
    #[test]
    #[ignore]
    fn trivial_ignored(n: u32) {
        assert!(n == 42)
    }
}
