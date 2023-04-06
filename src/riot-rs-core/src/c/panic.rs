pub use core::ffi::c_char;

#[no_mangle]
pub unsafe extern "C" fn _core_panic(_panic_code: usize, _msg: &'static c_char) -> ! {
    panic!("rust core_panic()");
}
