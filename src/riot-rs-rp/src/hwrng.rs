pub fn construct_rng(_peripherals: &mut crate::OptionalPeripherals) {
    riot_rs_random::construct_rng(embassy_rp::clocks::RoscRng);
}
