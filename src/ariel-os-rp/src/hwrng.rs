pub fn construct_rng(_peripherals: &mut crate::OptionalPeripherals) {
    ariel_os_random::construct_rng(embassy_rp::clocks::RoscRng);
}
